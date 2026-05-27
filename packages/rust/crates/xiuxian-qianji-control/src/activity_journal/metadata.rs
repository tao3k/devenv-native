//! LLM activity journal metadata helpers.

use crate::{ActivityTask, LlmActivityAdmission};

const LLM_ACTIVITY_REQUEST_AUDIT_METADATA_KEY: &str = "qianji_llm_activity_request";
const ORIGINAL_ACTIVITY_METADATA_KEY: &str = "qianji_original_activity_metadata";

pub(super) fn llm_activity_schedule_task(admission: &LlmActivityAdmission) -> ActivityTask {
    let mut task = admission.activity_task().clone();
    task.metadata =
        with_llm_request_audit_metadata(task.metadata, llm_request_audit_metadata(admission));
    task
}

fn with_llm_request_audit_metadata(
    existing_metadata: serde_json::Value,
    audit_metadata: serde_json::Value,
) -> serde_json::Value {
    match existing_metadata {
        serde_json::Value::Object(mut metadata) => {
            metadata.insert(
                LLM_ACTIVITY_REQUEST_AUDIT_METADATA_KEY.to_owned(),
                audit_metadata,
            );
            serde_json::Value::Object(metadata)
        }
        serde_json::Value::Null => serde_json::json!({
            LLM_ACTIVITY_REQUEST_AUDIT_METADATA_KEY: audit_metadata,
        }),
        metadata => serde_json::json!({
            LLM_ACTIVITY_REQUEST_AUDIT_METADATA_KEY: audit_metadata,
            ORIGINAL_ACTIVITY_METADATA_KEY: metadata,
        }),
    }
}

fn llm_request_audit_metadata(admission: &LlmActivityAdmission) -> serde_json::Value {
    let request = &admission.activity.request;
    serde_json::json!({
        "schema": "qianji.llm_activity_request_audit.v1",
        "model": request.model.as_str(),
        "prompt_ref": &request.prompt_ref,
        "context_ref": &request.context_ref,
        "tool_schema_hash": &request.tool_schema_hash,
        "temperature_millis": request.temperature_millis,
        "max_tokens": request.max_tokens,
        "response_schema_ref": &request.response_schema_ref,
        "budget": &request.budget,
        "request_metadata": &request.metadata,
        "admission_metadata": &admission.metadata,
    })
}
