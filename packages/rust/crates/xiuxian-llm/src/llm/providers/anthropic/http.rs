//! Anthropic messages HTTP transport and retry handling.

use std::time::Duration;

use serde_json::Value;
use tokio::time::sleep;
use tracing::warn;

use crate::llm::error::{LlmError, LlmResult, sanitize_user_visible};

/// HTTP request context for Anthropic-compatible messages calls.
#[derive(Debug, Clone, Copy)]
pub struct AnthropicMessagesHttpRequest<'a> {
    /// Shared HTTP client.
    pub client: &'a reqwest::Client,
    /// Provider endpoint URL.
    pub endpoint: &'a str,
    /// Provider API key.
    pub api_key: &'a str,
    /// JSON request body.
    pub body: &'a Value,
    /// Maximum transport attempts.
    pub attempts: usize,
}

/// Send Anthropic-compatible `messages` request with retry on transient transport errors.
///
/// # Errors
///
/// Returns `LlmError::Internal` when all retry attempts fail due to network transport issues.
/// Returns `LlmError::ConnectionFailed` when request building/sending fails on a non-retryable error.
pub async fn send_anthropic_messages_with_retry(
    request: AnthropicMessagesHttpRequest<'_>,
) -> LlmResult<reqwest::Response> {
    send_anthropic_messages_with_retry_impl(request).await
}

async fn send_anthropic_messages_with_retry_impl(
    request: AnthropicMessagesHttpRequest<'_>,
) -> LlmResult<reqwest::Response> {
    let max_attempts = request.attempts.max(1);
    let mut attempt = 1usize;
    loop {
        let result = request
            .client
            .post(request.endpoint)
            .header("x-api-key", request.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(request.body)
            .send()
            .await;

        match result {
            Ok(response) => return Ok(response),
            Err(error) => {
                let retryable = is_retryable_network_error(&error);
                if !retryable || attempt >= max_attempts {
                    if retryable {
                        return Err(LlmError::Internal {
                            message: format!(
                                "anthropic request network error after {attempt}/{max_attempts} attempt(s): {error}"
                            ),
                        });
                    }
                    return Err(LlmError::ConnectionFailed { source: error });
                }
                let backoff = retry_backoff_for_attempt(attempt);
                warn!(
                    event = "xiuxian.llm.providers.anthropic_http.network_retry",
                    endpoint = request.endpoint,
                    attempt,
                    max_attempts,
                    backoff_ms = backoff.as_millis(),
                    error = %error,
                    "Anthropic request hit transient network error; retrying"
                );
                sleep(backoff).await;
                attempt = attempt.saturating_add(1);
            }
        }
    }
}

/// Send Anthropic-compatible `messages` request and decode JSON body.
///
/// # Errors
///
/// Returns `LlmError::Internal` when the endpoint returns a non-success status or
/// when the response payload cannot be decoded as JSON.
pub async fn send_anthropic_messages_json_with_retry(
    request: AnthropicMessagesHttpRequest<'_>,
) -> LlmResult<Value> {
    let response = send_anthropic_messages_with_retry(request).await?;
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.map_err(|source| LlmError::Internal {
            message: format!(
                "anthropic response read failed after HTTP {status}: {}",
                sanitize_user_visible(&source.to_string())
            ),
        })?;
        return Err(LlmError::Internal {
            message: format!(
                "anthropic request failed with HTTP {status}: {}",
                sanitize_user_visible(&error_text)
            ),
        });
    }

    response.json().await.map_err(|source| LlmError::Internal {
        message: format!(
            "anthropic response json decode failed: {}",
            sanitize_user_visible(&source.to_string())
        ),
    })
}

fn retry_backoff_for_attempt(attempt: usize) -> Duration {
    match attempt {
        1 => Duration::from_millis(250),
        2 => Duration::from_millis(500),
        _ => Duration::from_secs(1),
    }
}

fn is_retryable_network_error(error: &reqwest::Error) -> bool {
    if error.is_connect() || error.is_timeout() {
        return true;
    }
    let text = error.to_string().to_ascii_lowercase();
    text.contains("error sending request")
        || text.contains("connection reset")
        || text.contains("connection aborted")
        || text.contains("connection closed")
        || text.contains("broken pipe")
}
