//! OpenAI-compatible activity executor side-effect adapter.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;
use xiuxian_qianji_control::{ArtifactRef, ErrorCode, WorkerActivityTask};

use crate::qianji_cli::invalid_input;

use super::activity_artifact::{ActivityOutputArtifactRequest, write_activity_output_artifact};
use super::activity_executor::ActivityExecutorOutcome;

const AUDIT_METADATA_KEY: &str = "qianji_llm_activity_request";
const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const RESPONSE_SCHEMA: &str = "qianji.openai_compatible_llm_response.v1";

#[derive(Clone, Copy)]
pub(crate) struct OpenAiCompatibleLlmExecutionRequest<'a> {
    pub(crate) task: &'a WorkerActivityTask,
    pub(crate) base_url: &'a str,
    pub(crate) api_key: Option<&'a str>,
    pub(crate) timeout_ms: Option<u64>,
    pub(crate) output_artifact_path: &'a Path,
    pub(crate) output_artifact_id: Option<&'a str>,
    pub(crate) output_artifact_kind: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct LlmRequestAudit {
    model: String,
    prompt_ref: ArtifactRef,
    #[serde(default)]
    context_ref: Option<ArtifactRef>,
    #[serde(default)]
    temperature_millis: Option<u32>,
    #[serde(default)]
    max_tokens: Option<u32>,
}

pub(crate) async fn execute_openai_compatible_llm(
    request: &OpenAiCompatibleLlmExecutionRequest<'_>,
) -> io::Result<ActivityExecutorOutcome> {
    let audit = match llm_request_audit(request.task) {
        Ok(audit) => audit,
        Err(error) => {
            return provider_failure(
                "request_audit_invalid",
                format!("OpenAI-compatible LLM request audit was invalid: {error}"),
                false,
                serde_json::json!({
                    "executor": "openai-compatible-llm",
                    "error": error.to_string(),
                }),
            );
        }
    };
    let payload = match openai_chat_payload(&audit) {
        Ok(payload) => payload,
        Err(error) => {
            return provider_failure(
                "input_artifact_read_failed",
                format!("OpenAI-compatible LLM input artifact read failed: {error}"),
                false,
                serde_json::json!({
                    "executor": "openai-compatible-llm",
                    "model": audit.model,
                    "error": error.to_string(),
                }),
            );
        }
    };
    let body = match fetch_openai_chat_completion(request, &audit, &payload).await? {
        Ok(body) => body,
        Err(outcome) => return Ok(outcome),
    };
    let provider_response: Value = match serde_json::from_str(&body) {
        Ok(response) => response,
        Err(error) => {
            return provider_failure(
                "provider_malformed_response",
                format!("OpenAI-compatible LLM response was not valid JSON: {error}"),
                false,
                serde_json::json!({
                    "executor": "openai-compatible-llm",
                    "model": audit.model,
                    "body_preview": body_preview(&body),
                }),
            );
        }
    };
    let Some(response_text) = openai_message_content(&provider_response) else {
        return provider_failure(
            "provider_malformed_response",
            "OpenAI-compatible LLM response did not include choices[0].message.content",
            false,
            serde_json::json!({
                "executor": "openai-compatible-llm",
                "model": audit.model,
                "body_preview": body_preview(&body),
            }),
        );
    };
    let response_text = response_text.to_owned();
    let response_chars = response_text.chars().count();
    let artifact = write_openai_response_artifact(
        request,
        &audit,
        response_text.as_str(),
        &provider_response,
    )?;
    Ok(ActivityExecutorOutcome::Complete {
        result: xiuxian_qianji_control::ActivityResult {
            output_ref: Some(artifact.output_ref),
            output_hash: Some(artifact.output_hash),
            metadata: serde_json::json!({
                "executor": "openai-compatible-llm",
                "model": audit.model,
                "response_chars": response_chars,
            }),
        },
    })
}

async fn fetch_openai_chat_completion(
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
    if let Some(api_key) = request.api_key.filter(|api_key| !api_key.trim().is_empty()) {
        builder = builder.bearer_auth(api_key);
    }
    let response = match builder.send().await {
        Ok(response) => response,
        Err(error) => {
            return provider_failure(
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
    provider_failure(
        "provider_http_error",
        format!("OpenAI-compatible LLM request returned HTTP {status}"),
        true,
        serde_json::json!({
            "executor": "openai-compatible-llm",
            "model": audit.model,
            "http_status": status.as_u16(),
            "body_preview": body_preview(&body),
        }),
    )
    .map(Err)
}

fn openai_message_content(provider_response: &Value) -> Option<&str> {
    provider_response
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .filter(|content| !content.trim().is_empty())
}

fn write_openai_response_artifact(
    request: &OpenAiCompatibleLlmExecutionRequest<'_>,
    audit: &LlmRequestAudit,
    response_text: &str,
    provider_response: &Value,
) -> io::Result<super::activity_artifact::ActivityOutputArtifact> {
    let artifact_content = serde_json::to_string_pretty(&serde_json::json!({
        "schema": RESPONSE_SCHEMA,
        "model": audit.model,
        "activity_id": request.task.activity_id.as_str(),
        "content": response_text,
        "provider_response": provider_response,
    }))
    .map_err(io::Error::other)?;
    write_activity_output_artifact(
        request.task,
        ActivityOutputArtifactRequest {
            path: request.output_artifact_path,
            content: &artifact_content,
            artifact_id: request.output_artifact_id,
            artifact_kind: request.output_artifact_kind,
        },
    )
}

fn llm_request_audit(task: &WorkerActivityTask) -> io::Result<LlmRequestAudit> {
    let audit = task.metadata.get(AUDIT_METADATA_KEY).ok_or_else(|| {
        invalid_input(format!(
            "activity executor `openai-compatible-llm` requires `{AUDIT_METADATA_KEY}` metadata"
        ))
    })?;
    serde_json::from_value(audit.clone()).map_err(|error| {
        invalid_input(format!(
            "activity executor `openai-compatible-llm` has invalid request audit metadata: {error}"
        ))
    })
}

fn openai_chat_payload(audit: &LlmRequestAudit) -> io::Result<Value> {
    let prompt = read_local_artifact_text(&audit.prompt_ref)?;
    let mut content = prompt;
    if let Some(context_ref) = &audit.context_ref {
        let context_text = read_local_artifact_text(context_ref)?;
        content.push_str("\n\n<context>\n");
        content.push_str(&context_text);
        content.push_str("\n</context>");
    }
    let mut payload = serde_json::json!({
        "model": audit.model,
        "messages": [
            {
                "role": "user",
                "content": content
            }
        ]
    });
    if let Some(temperature_millis) = audit.temperature_millis {
        payload["temperature"] = serde_json::json!(f64::from(temperature_millis) / 1000.0_f64);
    }
    if let Some(max_tokens) = audit.max_tokens {
        payload["max_tokens"] = serde_json::json!(max_tokens);
    }
    Ok(payload)
}

fn read_local_artifact_text(artifact_ref: &ArtifactRef) -> io::Result<String> {
    let uri = artifact_ref.uri.trim();
    if uri.is_empty() {
        return Err(invalid_input("LLM artifact URI must not be blank"));
    }
    if uri.starts_with("artifact://") || uri.starts_with("http://") || uri.starts_with("https://") {
        return Err(invalid_input(format!(
            "OpenAI-compatible executor can only materialize local file artifacts in this slice, got `{uri}`"
        )));
    }
    fs::read_to_string(local_artifact_path(uri)).map_err(|error| {
        io::Error::other(format!(
            "failed to read LLM local artifact `{uri}`: {error}"
        ))
    })
}

fn local_artifact_path(uri: &str) -> PathBuf {
    if let Some(path) = uri.strip_prefix("file://") {
        return PathBuf::from(path);
    }
    PathBuf::from(uri)
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

fn provider_failure(
    error_code: &'static str,
    message: impl Into<String>,
    retryable: bool,
    metadata: Value,
) -> io::Result<ActivityExecutorOutcome> {
    Ok(ActivityExecutorOutcome::Fail {
        error_code: ErrorCode::new(error_code)
            .map_err(|error| invalid_input(format!("{error}")))?,
        message: message.into(),
        retryable,
        metadata,
    })
}

fn body_preview(body: &str) -> String {
    body.chars().take(4096).collect()
}
