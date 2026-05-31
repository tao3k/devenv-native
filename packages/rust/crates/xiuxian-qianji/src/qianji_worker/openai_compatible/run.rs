use std::io;

use serde_json::Value;
use xiuxian_llm::llm::ChatRequest;
use xiuxian_qianji_control::WorkerActivityTask;

use super::artifact;
use super::episteme;
use super::failure;
use super::io_support;
use super::protocol::{AUDIT_METADATA_KEY, EPISTEME_REASONING_ACTIVITY_TYPE};
use super::response;
use super::transport;
use super::types::{LlmRequestAudit, OpenAiCompatibleLlmExecutionRequest};
use crate::qianji_worker::{ActivityExecutorOutcome, invalid_input};

struct OpenAiChatPayload {
    request: ChatRequest,
    episteme_context: Option<episteme::EpistemeReasoningContextExpectation>,
}

pub(crate) async fn execute_openai_compatible_llm(
    request: &OpenAiCompatibleLlmExecutionRequest<'_>,
) -> io::Result<ActivityExecutorOutcome> {
    let audit = match request_audit_or_failure(request.task)? {
        Ok(audit) => audit,
        Err(outcome) => return Ok(outcome),
    };
    let chat_payload = match chat_payload_or_failure(request.task, &audit)? {
        Ok(payload) => payload,
        Err(outcome) => return Ok(outcome),
    };
    let body = match transport::fetch_openai_chat_completion(request, &audit, chat_payload.request)
        .await?
    {
        Ok(body) => body,
        Err(outcome) => return Ok(outcome),
    };
    let provider_response = match provider_response_or_failure(&body, &audit)? {
        Ok(response) => response,
        Err(outcome) => return Ok(outcome),
    };
    let response_text = match response_text_or_failure(&body, &audit, &provider_response)? {
        Ok(response_text) => response_text,
        Err(outcome) => return Ok(outcome),
    };
    let episteme_review = match episteme_review_or_failure(
        &body,
        &audit,
        &provider_response,
        &response_text,
        chat_payload.episteme_context.as_ref(),
    )? {
        Ok(review) => review,
        Err(outcome) => return Ok(outcome),
    };
    let response_chars = response_text.chars().count();
    let response_preview = response::response_preview(response_text.as_str());
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
                "response_preview": response_preview,
            }),
        },
    })
}

fn request_audit_or_failure(
    task: &WorkerActivityTask,
) -> io::Result<Result<LlmRequestAudit, ActivityExecutorOutcome>> {
    match llm_request_audit(task) {
        Ok(audit) => Ok(Ok(audit)),
        Err(error) => failure::provider_failure(
            "request_audit_invalid",
            format!("OpenAI-compatible LLM request audit was invalid: {error}"),
            false,
            serde_json::json!({
                "executor": "openai-compatible-llm",
                "error": error.to_string(),
            }),
        )
        .map(Err),
    }
}

fn chat_payload_or_failure(
    task: &WorkerActivityTask,
    audit: &LlmRequestAudit,
) -> io::Result<Result<OpenAiChatPayload, ActivityExecutorOutcome>> {
    match openai_chat_payload(task, audit) {
        Ok(payload) => Ok(Ok(payload)),
        Err(error) => failure::provider_failure(
            "input_artifact_read_failed",
            format!("OpenAI-compatible LLM input artifact read failed: {error}"),
            false,
            serde_json::json!({
                "executor": "openai-compatible-llm",
                "model": audit.model,
                "error": error.to_string(),
            }),
        )
        .map(Err),
    }
}

fn provider_response_or_failure(
    body: &str,
    audit: &LlmRequestAudit,
) -> io::Result<Result<Value, ActivityExecutorOutcome>> {
    match serde_json::from_str(body) {
        Ok(response) => Ok(Ok(response)),
        Err(error) => failure::provider_failure(
            "provider_malformed_response",
            format!("OpenAI-compatible LLM response was not valid JSON: {error}"),
            true,
            serde_json::json!({
                "executor": "openai-compatible-llm",
                "model": audit.model,
                "body_preview": response::body_preview(body),
            }),
        )
        .map(Err),
    }
}

fn response_text_or_failure(
    body: &str,
    audit: &LlmRequestAudit,
    provider_response: &Value,
) -> io::Result<Result<String, ActivityExecutorOutcome>> {
    match response::openai_message_content(provider_response) {
        Some(response_text) => Ok(Ok(response_text.to_owned())),
        None => failure::provider_failure(
            "provider_malformed_response",
            "OpenAI-compatible LLM response did not include choices[0].message.content",
            true,
            serde_json::json!({
                "executor": "openai-compatible-llm",
                "model": audit.model,
                "body_preview": response::body_preview(body),
            }),
        )
        .map(Err),
    }
}

fn episteme_review_or_failure(
    body: &str,
    audit: &LlmRequestAudit,
    provider_response: &Value,
    response_text: &str,
    episteme_context: Option<&episteme::EpistemeReasoningContextExpectation>,
) -> io::Result<Result<Option<Value>, ActivityExecutorOutcome>> {
    match episteme::episteme_reasoning_review_json(response_text, episteme_context) {
        Ok(review) => Ok(Ok(review)),
        Err(message) => failure::provider_failure(
            "provider_contract_invalid",
            message.clone(),
            response::retryable_contract_invalid(&message, provider_response),
            serde_json::json!({
                "executor": "openai-compatible-llm",
                "model": audit.model,
                "body_preview": response::body_preview(body),
            }),
        )
        .map(Err),
    }
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
    let mut request = ChatRequest::new(audit.model.clone()).add_user_message(content);
    if let Some(temperature_millis) = audit.temperature_millis {
        let bounded_temperature_millis = u16::try_from(temperature_millis).map_err(|_| {
            invalid_input("LLM temperature_millis must fit in u16 before f32 conversion")
        })?;
        request = request.with_temperature(f32::from(bounded_temperature_millis) / 1000.0_f32);
    }
    if let Some(max_tokens) = audit.max_tokens {
        request = request.with_max_tokens(max_tokens);
    }
    Ok(OpenAiChatPayload {
        request,
        episteme_context,
    })
}
