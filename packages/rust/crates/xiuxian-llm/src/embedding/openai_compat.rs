//! OpenAI-compatible embedding request and response utilities.

use std::time::{Duration, Instant};

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;

const OPENAI_HTTP_RETRY_BASE_DELAY_MS: u64 = 80;
const OPENAI_HTTP_RETRY_MAX_DELAY_MS: u64 = 1_000;
const OPENAI_HTTP_MAX_ATTEMPTS: usize = 8;
const RESPONSE_BODY_PREVIEW_LIMIT: usize = 160;

enum AttemptOutcome<T> {
    Retry,
    ReturnNone,
    Value(T),
}

#[derive(Deserialize)]
struct OpenAiEmbeddingsResponse {
    #[serde(default)]
    data: Vec<OpenAiEmbeddingItem>,
}

#[derive(Deserialize)]
struct OpenAiEmbeddingItem {
    embedding: Vec<f32>,
}

/// Normalize upstream base URL to a concrete OpenAI-compatible embeddings endpoint.
#[must_use]
pub fn normalize_openai_embeddings_url(base_url: &str) -> Option<String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.ends_with("/v1") {
        return Some(format!("{trimmed}/embeddings"));
    }
    Some(format!("{trimmed}/v1/embeddings"))
}

/// Embed a batch through an OpenAI-compatible `/v1/embeddings` API.
///
/// Returns `None` on network/HTTP/parse failures so callers can apply fallback policy.
pub async fn embed_openai_compatible(
    client: &Client,
    base_url: &str,
    texts: &[String],
    model: Option<&str>,
) -> Option<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Some(vec![]);
    }
    let url = normalize_openai_embeddings_url(base_url)?;
    let started = Instant::now();
    let body = build_openai_request_body(texts, model);
    for attempt in 1..=OPENAI_HTTP_MAX_ATTEMPTS {
        let resp = match send_openai_request(client, &url, &body, started, attempt).await {
            AttemptOutcome::Retry => continue,
            AttemptOutcome::ReturnNone => return None,
            AttemptOutcome::Value(resp) => resp,
        };
        match handle_openai_response(resp, started, attempt).await {
            AttemptOutcome::Retry => {}
            AttemptOutcome::ReturnNone => return None,
            AttemptOutcome::Value(vectors) => return Some(vectors),
        }
    }
    None
}

fn build_openai_request_body(texts: &[String], model: Option<&str>) -> serde_json::Value {
    let mut body = serde_json::json!({ "input": texts });
    if let Some(model) = model.map(str::trim).filter(|value| !value.is_empty()) {
        body["model"] = serde_json::Value::String(model.to_string());
    }
    body
}

async fn send_openai_request(
    client: &Client,
    url: &str,
    body: &serde_json::Value,
    started: Instant,
    attempt: usize,
) -> AttemptOutcome<reqwest::Response> {
    match client.post(url).json(body).send().await {
        Ok(resp) => AttemptOutcome::Value(resp),
        Err(error) => {
            let should_retry =
                attempt < OPENAI_HTTP_MAX_ATTEMPTS && should_retry_http_request_error(&error);
            if should_retry {
                let retry_delay = retry_delay_for_attempt(attempt);
                tracing::debug!(
                    event = "xiuxian.llm.embedding.openai_http.request_retry",
                    url,
                    attempt,
                    max_attempts = OPENAI_HTTP_MAX_ATTEMPTS,
                    retry_delay_ms = retry_delay.as_millis(),
                    elapsed_ms = started.elapsed().as_millis(),
                    error = %error,
                    "embedding openai-compatible request failed; retrying"
                );
                sleep_before_retry(retry_delay).await;
                AttemptOutcome::Retry
            } else {
                tracing::debug!(
                    event = "xiuxian.llm.embedding.openai_http.request_failed",
                    url,
                    attempt,
                    max_attempts = OPENAI_HTTP_MAX_ATTEMPTS,
                    elapsed_ms = started.elapsed().as_millis(),
                    error = %error,
                    "embedding openai-compatible request failed"
                );
                AttemptOutcome::ReturnNone
            }
        }
    }
}

async fn handle_openai_response(
    resp: reqwest::Response,
    started: Instant,
    attempt: usize,
) -> AttemptOutcome<Vec<Vec<f32>>> {
    let status = resp.status();
    let content_type = resp
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();

    if !status.is_success() {
        return handle_non_success_status(resp, status, content_type, started, attempt).await;
    }

    read_success_vectors(resp, content_type, started, attempt).await
}

async fn handle_non_success_status(
    resp: reqwest::Response,
    status: reqwest::StatusCode,
    content_type: String,
    started: Instant,
    attempt: usize,
) -> AttemptOutcome<Vec<Vec<f32>>> {
    let should_retry = attempt < OPENAI_HTTP_MAX_ATTEMPTS && status.is_server_error();
    if should_retry {
        let retry_delay = retry_delay_for_attempt(attempt);
        tracing::debug!(
            event = "xiuxian.llm.embedding.openai_http.retry_on_server_error",
            status = %status,
            attempt,
            max_attempts = OPENAI_HTTP_MAX_ATTEMPTS,
            retry_delay_ms = retry_delay.as_millis(),
            elapsed_ms = started.elapsed().as_millis(),
            "embedding openai-compatible returned server error; retrying"
        );
        sleep_before_retry(retry_delay).await;
        return AttemptOutcome::Retry;
    }

    let body_preview = read_response_preview(resp).await;
    tracing::debug!(
        event = "xiuxian.llm.embedding.openai_http.non_success_status",
        status = %status,
        attempt,
        max_attempts = OPENAI_HTTP_MAX_ATTEMPTS,
        elapsed_ms = started.elapsed().as_millis(),
        body_preview = body_preview.as_deref().unwrap_or(""),
        content_type = content_type.as_str(),
        "embedding openai-compatible returned non-success status"
    );
    AttemptOutcome::ReturnNone
}

async fn read_success_vectors(
    resp: reqwest::Response,
    content_type: String,
    started: Instant,
    attempt: usize,
) -> AttemptOutcome<Vec<Vec<f32>>> {
    let response_body = match resp.text().await {
        Ok(body_text) => body_text,
        Err(error) => {
            tracing::debug!(
                event = "xiuxian.llm.embedding.openai_http.decode_failed",
                elapsed_ms = started.elapsed().as_millis(),
                attempt,
                max_attempts = OPENAI_HTTP_MAX_ATTEMPTS,
                error = %error,
                "embedding openai-compatible response body read failed"
            );
            return AttemptOutcome::ReturnNone;
        }
    };
    let body_preview = preview_text(&response_body, RESPONSE_BODY_PREVIEW_LIMIT);
    let data: OpenAiEmbeddingsResponse = match serde_json::from_str(&response_body) {
        Ok(data) => data,
        Err(error) => {
            tracing::debug!(
                event = "xiuxian.llm.embedding.openai_http.decode_failed",
                elapsed_ms = started.elapsed().as_millis(),
                attempt,
                max_attempts = OPENAI_HTTP_MAX_ATTEMPTS,
                error = %error,
                body_preview = body_preview.as_str(),
                content_type = content_type.as_str(),
                "embedding openai-compatible response decode failed"
            );
            if !is_json_content_type(&content_type) {
                tracing::warn!(
                    event = "xiuxian.llm.embedding.openai_http.unexpected_content_type",
                    content_type = content_type.as_str(),
                    body_preview = body_preview.as_str(),
                    "embedding openai-compatible returned non-JSON content type"
                );
            }
            return AttemptOutcome::ReturnNone;
        }
    };
    let vectors = data
        .data
        .into_iter()
        .map(|item| item.embedding)
        .collect::<Vec<_>>();
    tracing::debug!(
        event = "xiuxian.llm.embedding.openai_http.completed",
        elapsed_ms = started.elapsed().as_millis(),
        attempt,
        max_attempts = OPENAI_HTTP_MAX_ATTEMPTS,
        success = true,
        vector_count = vectors.len(),
        "embedding openai-compatible path completed"
    );
    AttemptOutcome::Value(vectors)
}

fn should_retry_http_request_error(error: &reqwest::Error) -> bool {
    error.is_connect()
        || error.is_timeout()
        || error.to_string().contains("error sending request for url")
}

fn retry_delay_for_attempt(attempt: usize) -> Duration {
    let attempt_index = attempt.saturating_sub(1);
    let shift = u32::try_from(attempt_index.min(6)).unwrap_or(6);
    let factor = 1_u64.checked_shl(shift).unwrap_or(u64::MAX);
    let delay_ms = OPENAI_HTTP_RETRY_BASE_DELAY_MS.saturating_mul(factor);
    Duration::from_millis(delay_ms.min(OPENAI_HTTP_RETRY_MAX_DELAY_MS))
}

async fn sleep_before_retry(delay: Duration) {
    tokio::time::sleep(delay).await;
}

async fn read_response_preview(resp: reqwest::Response) -> Option<String> {
    match resp.text().await {
        Ok(body) => Some(preview_text(&body, RESPONSE_BODY_PREVIEW_LIMIT)),
        Err(_) => None,
    }
}

fn preview_text(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let mut out = String::new();
    for ch in text.chars().take(max_chars) {
        out.push(ch);
    }
    out
}

fn is_json_content_type(content_type: &str) -> bool {
    let normalized = content_type.trim().to_ascii_lowercase();
    normalized.contains("application/json")
        || normalized.ends_with("+json")
        || normalized.contains("+json;")
}
