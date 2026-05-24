//! OpenAI-compatible activity executor side-effect adapter.

use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{borrow::Cow, fs};

use serde::Deserialize;
use serde_json::Value;
use xiuxian_qianji_control::{ArtifactRef, ErrorCode, WorkerActivityTask};

use crate::qianji_cli::invalid_input;

use super::activity_artifact::{ActivityOutputArtifactRequest, write_activity_output_artifact};
use super::activity_executor::ActivityExecutorOutcome;

const AUDIT_METADATA_KEY: &str = "qianji_llm_activity_request";
const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const RESPONSE_SCHEMA: &str = "qianji.openai_compatible_llm_response.v1";
const EPISTEME_REASONING_ACTIVITY_TYPE: &str = "episteme.ontology.reasoning_fill";
const EPISTEME_REASONING_CONTEXT_KIND: &str = "episteme.reasoning_fill_context";
const EPISTEME_REASONING_CONTEXT_SCHEMA: &str = "xiuxian.wendao.episteme.reasoning_fill_context.v1";
const EPISTEME_REASONING_TARGET_CONTRACT_SCHEMA: &str =
    "xiuxian.wendao.episteme.reasoning_target_contract.v1";
const EPISTEME_REASONING_REVIEW_SCHEMA: &str = "xiuxian.wendao.episteme.reasoning_fill_review.v1";
const EPISTEME_OBJECT_MODEL_COMPATIBILITY: &str = "foundry_style_object_model_v1";
const EPISTEME_OBJECT_MODEL_TARGET_LAYER: &str = "object_model";
const EPISTEME_RDF_SOURCE_AUTHORITY: &str = "rdf";
const OBJECT_FIELD_GROUP: &str = "object_proposal";
const RELATION_FIELD_GROUP: &str = "relation_proposal";
const SERVICE_CATALOG_FIELD_GROUP: &str = "service_catalog_review";
const OBJECT_INSTANCE_FIELD_GROUP: &str = "object_instance_review";
const OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND: &str = "object_model_object_type_candidate";
const OBJECT_MODEL_LINK_TYPE_PATCH_KIND: &str = "object_model_link_type_candidate";
const OBJECT_CANDIDATE_PATCH_KIND: &str = "object_candidate";

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

struct OpenAiChatPayload {
    payload: Value,
    episteme_context: Option<EpistemeReasoningContextExpectation>,
}

struct EpistemeReasoningContextExpectation {
    fill_item_id: String,
    target_ledger_field_group: String,
    allowed_patch_kinds: Vec<String>,
}

struct EpistemeTargetContractExpectation {
    target_ledger_field_group: String,
    allowed_patch_kinds: Vec<String>,
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
    let chat_payload = match openai_chat_payload(request.task, &audit) {
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
    let body = match fetch_openai_chat_completion(request, &audit, &chat_payload.payload).await? {
        Ok(body) => body,
        Err(outcome) => return Ok(outcome),
    };
    let provider_response: Value = match serde_json::from_str(&body) {
        Ok(response) => response,
        Err(error) => {
            return provider_failure(
                "provider_malformed_response",
                format!("OpenAI-compatible LLM response was not valid JSON: {error}"),
                true,
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
            true,
            serde_json::json!({
                "executor": "openai-compatible-llm",
                "model": audit.model,
                "body_preview": body_preview(&body),
            }),
        );
    };
    let response_text = response_text.to_owned();
    let episteme_review = match episteme_reasoning_review_json(
        &response_text,
        chat_payload.episteme_context.as_ref(),
    ) {
        Ok(review) => review,
        Err(message) => {
            return provider_failure(
                "provider_contract_invalid",
                message.clone(),
                retryable_contract_invalid(&message, &provider_response),
                serde_json::json!({
                    "executor": "openai-compatible-llm",
                    "model": audit.model,
                    "body_preview": body_preview(&body),
                }),
            );
        }
    };
    let response_chars = response_text.chars().count();
    let artifact = write_openai_response_artifact(
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
    if let Some(api_key) = bearer_api_key(request.api_key) {
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

fn openai_message_content(provider_response: &Value) -> Option<&str> {
    provider_response
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .filter(|content| !content.trim().is_empty())
}

fn retryable_contract_invalid(message: &str, provider_response: &Value) -> bool {
    openai_finish_reason(provider_response).is_some_and(|reason| reason == "length")
        || message.contains("EOF while parsing")
}

fn openai_finish_reason(provider_response: &Value) -> Option<&str> {
    provider_response
        .pointer("/choices/0/finish_reason")
        .or_else(|| provider_response.pointer("/choices/0/native_finish_reason"))
        .and_then(Value::as_str)
}

fn write_openai_response_artifact(
    request: &OpenAiCompatibleLlmExecutionRequest<'_>,
    audit: &LlmRequestAudit,
    response_text: &str,
    episteme_review: Option<&Value>,
    provider_response: &Value,
) -> io::Result<super::activity_artifact::ActivityOutputArtifact> {
    let mut artifact = serde_json::json!({
        "schema": RESPONSE_SCHEMA,
        "model": audit.model,
        "activity_id": request.task.activity_id.as_str(),
        "content": response_text,
        "provider_response": provider_response,
    });
    if let Some(episteme_review) = episteme_review {
        artifact["episteme_review"] = episteme_review.clone();
    }
    let artifact_content = serde_json::to_string_pretty(&artifact).map_err(io::Error::other)?;
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

fn openai_chat_payload(
    task: &WorkerActivityTask,
    audit: &LlmRequestAudit,
) -> io::Result<OpenAiChatPayload> {
    let prompt = read_local_artifact_text(&audit.prompt_ref)?;
    let mut content = prompt;
    let mut episteme_context = None;
    if task.activity_type.as_str() == EPISTEME_REASONING_ACTIVITY_TYPE {
        let (context_text, expectation) = read_validated_episteme_reasoning_context(audit)?;
        content.push_str("\n\n<context>\n");
        content.push_str(&context_text);
        content.push_str("\n</context>");
        episteme_context = Some(expectation);
    } else if let Some(context_ref) = &audit.context_ref {
        let context_text = read_local_artifact_text(context_ref)?;
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

fn read_validated_episteme_reasoning_context(
    audit: &LlmRequestAudit,
) -> io::Result<(String, EpistemeReasoningContextExpectation)> {
    let context_ref = audit.context_ref.as_ref().ok_or_else(|| {
        invalid_input("Episteme ontology reasoning tasks require context_ref evidence")
    })?;
    if context_ref.artifact_kind.as_str() != EPISTEME_REASONING_CONTEXT_KIND {
        return Err(invalid_input(format!(
            "Episteme ontology reasoning context_ref must use artifact kind `{EPISTEME_REASONING_CONTEXT_KIND}`"
        )));
    }
    let context_text = read_local_artifact_text(context_ref)?;
    if context_text.trim().is_empty() {
        return Err(invalid_input(
            "Episteme ontology reasoning context artifact must not be empty",
        ));
    }
    let context_json: Value = serde_json::from_str(&context_text).map_err(|error| {
        invalid_input(format!(
            "Episteme ontology reasoning context artifact must be JSON: {error}"
        ))
    })?;
    let expectation = validate_episteme_reasoning_context_json(&context_json)?;
    Ok((context_text, expectation))
}

fn validate_episteme_reasoning_context_json(
    context: &Value,
) -> io::Result<EpistemeReasoningContextExpectation> {
    if context.get("schema").and_then(Value::as_str) != Some(EPISTEME_REASONING_CONTEXT_SCHEMA) {
        return Err(invalid_input(format!(
            "Episteme ontology reasoning context schema must be `{EPISTEME_REASONING_CONTEXT_SCHEMA}`"
        )));
    }
    let fill_item_id = context
        .pointer("/fillItem/fillItemId")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            invalid_input("Episteme ontology reasoning context requires fillItem.fillItemId")
        })?
        .to_owned();
    let evidence = context
        .get("contextEvidence")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            invalid_input("Episteme ontology reasoning context requires contextEvidence array")
        })?;
    if evidence.is_empty() {
        return Err(invalid_input(
            "Episteme ontology reasoning contextEvidence must not be empty",
        ));
    }
    let has_text = evidence.iter().any(|row| {
        row.get("extractedText")
            .and_then(Value::as_str)
            .is_some_and(|text| !text.trim().is_empty())
    });
    if !has_text {
        return Err(invalid_input(
            "Episteme ontology reasoning contextEvidence must include extractedText",
        ));
    }
    let target_contract = validate_episteme_reasoning_target_contract(context)?;
    Ok(EpistemeReasoningContextExpectation {
        fill_item_id,
        target_ledger_field_group: target_contract.target_ledger_field_group,
        allowed_patch_kinds: target_contract.allowed_patch_kinds,
    })
}

fn validate_episteme_reasoning_target_contract(
    context: &Value,
) -> io::Result<EpistemeTargetContractExpectation> {
    let target_contract = context.get("targetContract").ok_or_else(|| {
        invalid_input("Episteme ontology reasoning context requires targetContract")
    })?;
    if target_contract.get("schema").and_then(Value::as_str)
        != Some(EPISTEME_REASONING_TARGET_CONTRACT_SCHEMA)
    {
        return Err(invalid_input(format!(
            "Episteme ontology reasoning targetContract schema must be `{EPISTEME_REASONING_TARGET_CONTRACT_SCHEMA}`"
        )));
    }
    for field in ["targetLedgerFieldGroup", "patchKind", "reviewMode"] {
        if target_contract
            .get(field)
            .and_then(Value::as_str)
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(invalid_input(format!(
                "Episteme ontology reasoning targetContract requires non-empty {field}"
            )));
        }
    }
    if target_contract
        .get("objectModelCompatibility")
        .and_then(Value::as_str)
        != Some(EPISTEME_OBJECT_MODEL_COMPATIBILITY)
    {
        return Err(invalid_input(format!(
            "Episteme ontology reasoning targetContract objectModelCompatibility must be `{EPISTEME_OBJECT_MODEL_COMPATIBILITY}`"
        )));
    }
    if target_contract
        .get("operationalTargetLayer")
        .and_then(Value::as_str)
        != Some(EPISTEME_OBJECT_MODEL_TARGET_LAYER)
    {
        return Err(invalid_input(format!(
            "Episteme ontology reasoning targetContract operationalTargetLayer must be `{EPISTEME_OBJECT_MODEL_TARGET_LAYER}`"
        )));
    }
    if target_contract
        .get("semanticSourceAuthority")
        .and_then(Value::as_str)
        != Some(EPISTEME_RDF_SOURCE_AUTHORITY)
    {
        return Err(invalid_input(format!(
            "Episteme ontology reasoning targetContract semanticSourceAuthority must be `{EPISTEME_RDF_SOURCE_AUTHORITY}`"
        )));
    }
    if target_contract.get("reviewMode").and_then(Value::as_str) != Some("proposal_patch_only") {
        return Err(invalid_input(
            "Episteme ontology reasoning targetContract reviewMode must be `proposal_patch_only`",
        ));
    }
    for field in ["runtimeMutationAllowed", "rdfMutationAllowed"] {
        if target_contract.get(field).and_then(Value::as_bool) != Some(false) {
            return Err(invalid_input(format!(
                "Episteme ontology reasoning targetContract must keep {field}=false"
            )));
        }
    }
    if !target_contract
        .get("candidatePatchShape")
        .is_some_and(Value::is_object)
    {
        return Err(invalid_input(
            "Episteme ontology reasoning targetContract requires candidatePatchShape object",
        ));
    }
    let target_ledger_field_group = target_contract
        .get("targetLedgerFieldGroup")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let allowed_patch_kinds = allowed_patch_kinds(target_contract)?;
    validate_target_patch_shape(
        target_ledger_field_group.as_str(),
        &allowed_patch_kinds,
        &target_contract["candidatePatchShape"],
    )?;
    Ok(EpistemeTargetContractExpectation {
        target_ledger_field_group,
        allowed_patch_kinds,
    })
}

fn allowed_patch_kinds(target_contract: &Value) -> io::Result<Vec<String>> {
    let allowed = target_contract
        .get("allowedPatchKinds")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            invalid_input("Episteme ontology reasoning targetContract requires allowedPatchKinds")
        })?;
    if allowed.is_empty() {
        return Err(invalid_input(
            "Episteme ontology reasoning targetContract allowedPatchKinds must not be empty",
        ));
    }
    let mut kinds = Vec::with_capacity(allowed.len());
    for kind in allowed {
        let kind = kind
            .as_str()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                invalid_input(
                    "Episteme ontology reasoning targetContract allowedPatchKinds must be strings",
                )
            })?;
        if !is_supported_patch_kind(kind) {
            return Err(invalid_input(format!(
                "Episteme ontology reasoning targetContract unsupported patch kind `{kind}`"
            )));
        }
        kinds.push(kind.to_owned());
    }
    Ok(kinds)
}

fn validate_target_patch_shape(
    field_group: &str,
    allowed_patch_kinds: &[String],
    candidate_patch_shape: &Value,
) -> io::Result<()> {
    let patch_kind = candidate_patch_shape
        .get("patchKind")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            invalid_input(
                "Episteme ontology reasoning targetContract candidatePatchShape requires patchKind",
            )
        })?;
    if !allowed_patch_kinds
        .iter()
        .any(|allowed| allowed == patch_kind)
    {
        return Err(invalid_input(
            "Episteme ontology reasoning targetContract candidatePatchShape patchKind must be allowed",
        ));
    }
    match field_group {
        OBJECT_FIELD_GROUP
            if patch_kind == OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND
                && !candidate_patch_shape
                    .get("objectType")
                    .is_some_and(Value::is_object) =>
        {
            return Err(invalid_input(
                "Episteme ontology reasoning object targetContract requires objectType shape",
            ));
        }
        OBJECT_FIELD_GROUP if patch_kind == OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND => {}
        RELATION_FIELD_GROUP
            if patch_kind == OBJECT_MODEL_LINK_TYPE_PATCH_KIND
                && !candidate_patch_shape
                    .get("linkType")
                    .is_some_and(Value::is_object) =>
        {
            return Err(invalid_input(
                "Episteme ontology reasoning relation targetContract requires linkType shape",
            ));
        }
        RELATION_FIELD_GROUP if patch_kind == OBJECT_MODEL_LINK_TYPE_PATCH_KIND => {}
        OBJECT_FIELD_GROUP | RELATION_FIELD_GROUP => {
            return Err(invalid_input(
                "Episteme ontology reasoning targetContract patchKind does not match targetLedgerFieldGroup",
            ));
        }
        SERVICE_CATALOG_FIELD_GROUP | OBJECT_INSTANCE_FIELD_GROUP
            if patch_kind == OBJECT_CANDIDATE_PATCH_KIND =>
        {
            validate_concrete_object_candidate_shape(candidate_patch_shape)
                .map_err(invalid_input)?;
        }
        SERVICE_CATALOG_FIELD_GROUP | OBJECT_INSTANCE_FIELD_GROUP => {
            return Err(invalid_input(
                "Episteme ontology reasoning concrete-object targetContract requires object_candidate patchKind",
            ));
        }
        _ => {}
    }
    Ok(())
}

fn is_supported_patch_kind(kind: &str) -> bool {
    matches!(
        kind,
        OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND
            | OBJECT_MODEL_LINK_TYPE_PATCH_KIND
            | OBJECT_CANDIDATE_PATCH_KIND
    )
}

fn episteme_reasoning_review_json(
    response_text: &str,
    expectation: Option<&EpistemeReasoningContextExpectation>,
) -> Result<Option<Value>, String> {
    let Some(expectation) = expectation else {
        return Ok(None);
    };
    let review_text = strip_markdown_json_fence(response_text);
    let review: Value = serde_json::from_str(review_text.as_ref()).map_err(|error| {
        format!("Episteme ontology reasoning provider content must be JSON: {error}")
    })?;
    validate_episteme_reasoning_review_json(&review, expectation)?;
    Ok(Some(review))
}

fn validate_episteme_reasoning_review_json(
    review: &Value,
    expectation: &EpistemeReasoningContextExpectation,
) -> Result<(), String> {
    if review.get("schema").and_then(Value::as_str) != Some(EPISTEME_REASONING_REVIEW_SCHEMA) {
        return Err(format!(
            "Episteme ontology reasoning review schema must be `{EPISTEME_REASONING_REVIEW_SCHEMA}`"
        ));
    }
    if review.get("status").and_then(Value::as_str) != Some("review_only") {
        return Err("Episteme ontology reasoning review status must be `review_only`".to_owned());
    }
    if review.get("fillItemId").and_then(Value::as_str) != Some(expectation.fill_item_id.as_str()) {
        return Err("Episteme ontology reasoning review fillItemId mismatch".to_owned());
    }
    if review.get("targetLedgerFieldGroup").and_then(Value::as_str)
        != Some(expectation.target_ledger_field_group.as_str())
    {
        return Err(
            "Episteme ontology reasoning review targetLedgerFieldGroup mismatch".to_owned(),
        );
    }
    if review.get("rdfMutation").and_then(Value::as_bool) != Some(false) {
        return Err("Episteme ontology reasoning review must keep rdfMutation=false".to_owned());
    }
    let candidate_patches = review
        .get("candidatePatches")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            "Episteme ontology reasoning review requires candidatePatches array".to_owned()
        })?;
    let candidate_patch_count = review
        .get("candidatePatchCount")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            "Episteme ontology reasoning review requires integer candidatePatchCount".to_owned()
        })?;
    let candidate_patch_count = usize::try_from(candidate_patch_count).map_err(|_| {
        "Episteme ontology reasoning review candidatePatchCount exceeds platform bounds".to_owned()
    })?;
    if candidate_patch_count != candidate_patches.len() {
        return Err(
            "Episteme ontology reasoning review candidatePatchCount must match candidatePatches length"
                .to_owned(),
        );
    }
    for patch in candidate_patches {
        validate_episteme_candidate_patch_json(patch, expectation)?;
    }
    Ok(())
}

fn validate_episteme_candidate_patch_json(
    patch: &Value,
    expectation: &EpistemeReasoningContextExpectation,
) -> Result<(), String> {
    let patch_kind = patch
        .get("patchKind")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            "Episteme ontology reasoning candidate patch requires patchKind".to_owned()
        })?;
    if !expectation
        .allowed_patch_kinds
        .iter()
        .any(|allowed| allowed == patch_kind)
    {
        return Err(format!(
            "Episteme ontology reasoning candidate patch kind `{patch_kind}` is not allowed by targetContract"
        ));
    }
    if patch
        .get("fillItemId")
        .and_then(Value::as_str)
        .is_some_and(|value| value != expectation.fill_item_id)
    {
        return Err("Episteme ontology reasoning candidate patch fillItemId mismatch".to_owned());
    }
    if patch
        .get("targetLedgerFieldGroup")
        .and_then(Value::as_str)
        .is_some_and(|value| value != expectation.target_ledger_field_group)
    {
        return Err(
            "Episteme ontology reasoning candidate patch targetLedgerFieldGroup mismatch"
                .to_owned(),
        );
    }
    validate_candidate_source_evidence(patch)?;
    match patch_kind {
        OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND => validate_object_model_object_patch(patch),
        OBJECT_MODEL_LINK_TYPE_PATCH_KIND => validate_object_model_link_patch(patch),
        OBJECT_CANDIDATE_PATCH_KIND => validate_concrete_object_candidate_patch(patch),
        _ => Err(format!(
            "Episteme ontology reasoning unsupported patch kind `{patch_kind}`"
        )),
    }
}

fn validate_candidate_source_evidence(patch: &Value) -> Result<(), String> {
    let evidence = patch
        .get("sourceEvidence")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            "Episteme ontology reasoning candidate patch requires sourceEvidence array".to_owned()
        })?;
    if evidence.is_empty() {
        return Err(
            "Episteme ontology reasoning candidate patch sourceEvidence must not be empty"
                .to_owned(),
        );
    }
    let has_quote = evidence.iter().any(|row| {
        row.get("fileId")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
            && row
                .get("quote")
                .and_then(Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
    });
    if !has_quote {
        return Err(
            "Episteme ontology reasoning candidate patch sourceEvidence must cite fileId and quote"
                .to_owned(),
        );
    }
    Ok(())
}

fn validate_object_model_object_patch(patch: &Value) -> Result<(), String> {
    let object_type = patch
        .get("objectType")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            "Episteme ontology reasoning object model patch requires objectType object".to_owned()
        })?;
    for field in ["apiName", "displayName", "rdfClass"] {
        if object_type
            .get(field)
            .and_then(Value::as_str)
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(format!(
                "Episteme ontology reasoning objectType requires non-empty {field}"
            ));
        }
    }
    Ok(())
}

fn validate_object_model_link_patch(patch: &Value) -> Result<(), String> {
    let link_type = patch
        .get("linkType")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            "Episteme ontology reasoning object model patch requires linkType object".to_owned()
        })?;
    for field in [
        "apiName",
        "displayName",
        "rdfProperty",
        "fromObjectType",
        "toObjectType",
    ] {
        if link_type
            .get(field)
            .and_then(Value::as_str)
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(format!(
                "Episteme ontology reasoning linkType requires non-empty {field}"
            ));
        }
    }
    Ok(())
}

fn validate_concrete_object_candidate_shape(shape: &Value) -> Result<(), String> {
    for field in ["provisionalObjectKey", "label", "ontologyClassKey"] {
        if shape
            .get(field)
            .and_then(Value::as_str)
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(format!(
                "Episteme ontology reasoning object_candidate shape requires non-empty {field}"
            ));
        }
    }
    Ok(())
}

fn validate_concrete_object_candidate_patch(patch: &Value) -> Result<(), String> {
    validate_concrete_object_candidate_shape(patch)
}

fn strip_markdown_json_fence(response_text: &str) -> Cow<'_, str> {
    let trimmed = response_text.trim();
    let Some(rest) = trimmed.strip_prefix("```") else {
        return Cow::Borrowed(trimmed);
    };
    let Some(first_newline) = rest.find('\n') else {
        return Cow::Borrowed(trimmed);
    };
    let body = &rest[first_newline + 1..];
    let Some(fence_start) = body.rfind("```") else {
        return Cow::Borrowed(trimmed);
    };
    Cow::Owned(body[..fence_start].trim().to_owned())
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
