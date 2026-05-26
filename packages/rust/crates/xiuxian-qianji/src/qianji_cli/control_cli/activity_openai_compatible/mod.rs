//! OpenAI-compatible activity executor side-effect adapter.

mod artifact;
mod episteme;
mod failure;
mod io_support;
mod response;
mod transport;

use std::io;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;
use xiuxian_qianji_control::{ArtifactRef, WorkerActivityTask};

use crate::qianji_cli::invalid_input;

use super::activity_executor::ActivityExecutorOutcome;

const AUDIT_METADATA_KEY: &str = "qianji_llm_activity_request";
pub(super) const DEFAULT_TIMEOUT_MS: u64 = 30_000;
pub(super) const RESPONSE_SCHEMA: &str = "qianji.openai_compatible_llm_response.v1";
const EPISTEME_REASONING_ACTIVITY_TYPE: &str = "episteme.ontology.reasoning_fill";

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
pub(super) struct LlmRequestAudit {
    pub(super) model: String,
    pub(super) prompt_ref: ArtifactRef,
    #[serde(default)]
    pub(super) context_ref: Option<ArtifactRef>,
    #[serde(default)]
    pub(super) temperature_millis: Option<u32>,
    #[serde(default)]
    pub(super) max_tokens: Option<u32>,
}

struct OpenAiChatPayload {
    payload: Value,
    episteme_context: Option<episteme::EpistemeReasoningContextExpectation>,
}

pub(crate) async fn execute_openai_compatible_llm(
    request: &OpenAiCompatibleLlmExecutionRequest<'_>,
) -> io::Result<ActivityExecutorOutcome> {
    let audit = match llm_request_audit(request.task) {
        Ok(audit) => audit,
        Err(error) => {
            return failure::provider_failure(
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
    let chat_payload = match openai_chat_payload(request.task, &audit) {
        Ok(payload) => payload,
        Err(error) => {
            return failure::provider_failure(
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
    let body = match transport::fetch_openai_chat_completion(request, &audit, &chat_payload.payload)
        .await?
    {
        Ok(body) => body,
        Err(outcome) => return Ok(outcome),
    };
    let provider_response: Value = match serde_json::from_str(&body) {
        Ok(response) => response,
        Err(error) => {
            return failure::provider_failure(
                "provider_malformed_response",
                format!("OpenAI-compatible LLM response was not valid JSON: {error}"),
                true,
                serde_json::json!({
                    "executor": "openai-compatible-llm",
                    "model": audit.model,
                    "response::body_preview": response::body_preview(&body),
                }),
            );
        }
    };
    let Some(response_text) = response::openai_message_content(&provider_response) else {
        return failure::provider_failure(
            "provider_malformed_response",
            "OpenAI-compatible LLM response did not include choices[0].message.content",
            true,
            serde_json::json!({
                "executor": "openai-compatible-llm",
                "model": audit.model,
                "response::body_preview": response::body_preview(&body),
            }),
        );
    };
    let response_text = response_text.to_owned();
    let episteme_review = match episteme::episteme_reasoning_review_json(
        &response_text,
        chat_payload.episteme_context.as_ref(),
    ) {
        Ok(review) => review,
        Err(message) => {
            return failure::provider_failure(
                "provider_contract_invalid",
                message.clone(),
                response::retryable_contract_invalid(&message, &provider_response),
                serde_json::json!({
                    "executor": "openai-compatible-llm",
                    "model": audit.model,
                    "response::body_preview": response::body_preview(&body),
                }),
            );
        }
    };
    let response_chars = response_text.chars().count();
    let artifact = artifact::write_openai_response_artifact(
        request,
        &audit,
        response_text.as_str(),
        episteme_review.as_ref(),
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

fn openai_chat_payload(
    task: &WorkerActivityTask,
    audit: &LlmRequestAudit,
) -> io::Result<OpenAiChatPayload> {
    let prompt = io_support::read_local_artifact_text(&audit.prompt_ref)?;
    let mut content = prompt;
    let mut episteme_context = None;
    if task.activity_type.as_str() == EPISTEME_REASONING_ACTIVITY_TYPE {
        let (context_text, expectation) =
            episteme::read_validated_episteme_reasoning_context(audit)?;
        content.push_str("\n\n<context>\n");
        content.push_str(&context_text);
        content.push_str("\n</context>");
        episteme_context = Some(expectation);
    } else if let Some(context_ref) = &audit.context_ref {
        let context_text = io_support::read_local_artifact_text(context_ref)?;
        if context_text.trim().is_empty() {
            return Err(invalid_input("LLM context artifact text must not be empty"));
        }
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
    Ok(OpenAiChatPayload {
        payload,
        episteme_context,
    })
}
