use std::path::Path;

use serde::Deserialize;
use xiuxian_qianji_control::{ArtifactRef, WorkerActivityTask};

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
