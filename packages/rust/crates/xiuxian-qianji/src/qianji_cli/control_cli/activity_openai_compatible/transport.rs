use std::io;
use std::time::Duration;

use xiuxian_llm::llm::{ChatRequest, LlmError, OpenAICompatibleClient, OpenAIWireApi};

use super::failure;
use super::protocol::DEFAULT_TIMEOUT_MS;
use super::response;
use super::types::{LlmRequestAudit, OpenAiCompatibleLlmExecutionRequest};
use crate::qianji_cli::control_cli::ActivityExecutorOutcome;
use crate::qianji_cli::invalid_input;

pub(super) async fn fetch_openai_chat_completion(
    request: &OpenAiCompatibleLlmExecutionRequest<'_>,
    audit: &LlmRequestAudit,
    chat_request: ChatRequest,
) -> io::Result<Result<String, ActivityExecutorOutcome>> {
    let http = reqwest::Client::builder()
        .timeout(Duration::from_millis(
            request.timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS),
        ))
        .build()
        .map_err(io::Error::other)?;
    let client = OpenAICompatibleClient {
        api_key: api_key_for_client(request.api_key).unwrap_or_default(),
        base_url: chat_completions_base_url(request.base_url)?,
        wire_api: OpenAIWireApi::ChatCompletions,
        http,
    };
    match client.chat_completions_raw_body(chat_request).await {
        Ok(body) => Ok(Ok(body)),
        Err(error) => provider_error_to_outcome(audit, error).map(Err),
    }
}

fn provider_error_to_outcome(
    audit: &LlmRequestAudit,
    error: LlmError,
) -> io::Result<ActivityExecutorOutcome> {
    match error {
        LlmError::RequestFailed { status, reason, .. } => failure::provider_failure(
            "provider_http_error",
            format!("OpenAI-compatible LLM request returned HTTP {status}"),
            true,
            serde_json::json!({
                "executor": "openai-compatible-llm",
                "model": audit.model,
                "http_status": status.as_u16(),
                "body_preview": response::body_preview(&reason),
            }),
        ),
        LlmError::ResponseDecodingFailed {
            ref body_preview, ..
        } => failure::provider_failure(
            "provider_malformed_response",
            format!("OpenAI-compatible LLM response was not valid JSON: {error}"),
            true,
            serde_json::json!({
                "executor": "openai-compatible-llm",
                "model": audit.model,
                "body_preview": response::body_preview(body_preview),
            }),
        ),
        LlmError::EmptyTextChoice => failure::provider_failure(
            "provider_malformed_response",
            "OpenAI-compatible LLM response did not include choices[0].message.content",
            true,
            serde_json::json!({
                "executor": "openai-compatible-llm",
                "model": audit.model,
                "error": error.to_string(),
            }),
        ),
        _ => failure::provider_failure(
            "provider_request_failed",
            format!("OpenAI-compatible LLM request failed: {error}"),
            true,
            serde_json::json!({
                "executor": "openai-compatible-llm",
                "model": audit.model,
                "error": error.to_string(),
            }),
        ),
    }
}

fn api_key_for_client(api_key: Option<&str>) -> Option<String> {
    let trimmed = api_key?.trim();
    let unquoted = strip_matching_quotes(trimmed).trim();
    if unquoted.is_empty() {
        None
    } else {
        Some(unquoted.to_string())
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

fn chat_completions_base_url(base_url: &str) -> io::Result<String> {
    let trimmed = base_url.trim();
    if trimmed.is_empty() {
        return Err(invalid_input(
            "missing `--openai-compatible-base-url <url>` for `control activity-worker-once --executor openai-compatible-llm`",
        ));
    }
    if let Some(base) = trimmed.strip_suffix("/chat/completions") {
        return Ok(base.trim_end_matches('/').to_owned());
    }
    Ok(trimmed.trim_end_matches('/').to_owned())
}
