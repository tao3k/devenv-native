use std::io;

use serde_json::Value;

use super::{LlmRequestAudit, OpenAiCompatibleLlmExecutionRequest, RESPONSE_SCHEMA};
use crate::qianji_cli::control_cli::activity_artifact::{
    ActivityOutputArtifact, ActivityOutputArtifactRequest, write_activity_output_artifact,
};

pub(super) fn write_openai_response_artifact(
    request: &OpenAiCompatibleLlmExecutionRequest<'_>,
    audit: &LlmRequestAudit,
    response_text: &str,
    episteme_review: Option<&Value>,
    provider_response: &Value,
) -> io::Result<ActivityOutputArtifact> {
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
