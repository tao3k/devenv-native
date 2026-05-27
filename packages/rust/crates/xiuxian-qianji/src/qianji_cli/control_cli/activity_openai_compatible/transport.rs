use std::io;
use std::time::Duration;

use serde_json::Value;

use super::failure;
use super::protocol::DEFAULT_TIMEOUT_MS;
use super::response;
use super::types::{LlmRequestAudit, OpenAiCompatibleLlmExecutionRequest};
use crate::qianji_cli::control_cli::ActivityExecutorOutcome;
use crate::qianji_cli::invalid_input;

pub(super) async fn fetch_openai_chat_completion(
    request: &OpenAiCompatibleLlmExecutionRequest<'_>,
    audit: &LlmRequestAudit,
    payload: &Value,
) -> io::Result<Result<String, ActivityExecutorOutcome>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(
            request.timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS),
        ))
        .build()
        .map_err(io::Error::other)?;
    let endpoint = chat_completions_endpoint(request.base_url)?;
    let mut builder = client.post(endpoint).json(payload);
    if let Some(api_key) = bearer_api_key(request.api_key) {
        builder = builder.bearer_auth(api_key);
    }
    let response = match builder.send().await {
        Ok(response) => response,
        Err(error) => {
            return failure::provider_failure(
                "provider_request_failed",
                format!("OpenAI-compatible LLM request failed: {error}"),
                true,
                serde_json::json!({
                    "executor": "openai-compatible-llm",
                    "model": audit.model,
                    "error": error.to_string(),
                }),
            )
            .map(Err);
        }
    };
    let status = response.status();
    let body = response.text().await.map_err(io::Error::other)?;
    if status.is_success() {
        return Ok(Ok(body));
    }
    failure::provider_failure(
        "provider_http_error",
        format!("OpenAI-compatible LLM request returned HTTP {status}"),
        true,
        serde_json::json!({
            "executor": "openai-compatible-llm",
            "model": audit.model,
            "http_status": status.as_u16(),
            "body_preview": response::body_preview(&body),
        }),
    )
    .map(Err)
}

fn bearer_api_key(api_key: Option<&str>) -> Option<&str> {
    let trimmed = api_key?.trim();
    let unquoted = strip_matching_quotes(trimmed).trim();
    if unquoted.is_empty() {
        None
    } else {
        Some(unquoted)
    }
}

fn strip_matching_quotes(value: &str) -> &str {
    if value.len() < 2 {
        return value;
    }
    let bytes = value.as_bytes();
    let first = bytes[0];
    let last = bytes[bytes.len() - 1];
    if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn chat_completions_endpoint(base_url: &str) -> io::Result<String> {
    let trimmed = base_url.trim();
    if trimmed.is_empty() {
        return Err(invalid_input(
            "missing `--openai-compatible-base-url <url>` for `control activity-worker-once --executor openai-compatible-llm`",
        ));
    }
    if trimmed.ends_with("/chat/completions") {
        return Ok(trimmed.to_owned());
    }
    let endpoint_base = trimmed.trim_end_matches('/');
    Ok(format!("{endpoint_base}/chat/completions"))
}
