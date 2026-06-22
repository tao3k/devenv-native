//! Config data model for Qianji workflow-task routes.

use serde::{Deserialize, Serialize};

/// Qianji workflow-task LLM route profile loaded from `config/workflows`.
///
/// Semantic field boundary: this public DTO preserves externally serialized
/// TOML field names.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QianjiWorkflowLlmTaskConfig {
    /// Optional config schema marker.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    /// Provider/model endpoint overrides for this workflow-task profile.
    #[serde(default)]
    pub llm: QianjiWorkflowLlmEndpointConfig,
    /// Activity-task route contract for this workflow-task profile.
    #[serde(default)]
    pub task: QianjiWorkflowLlmTaskRouteConfig,
}

/// Secret-free LLM endpoint overrides scoped to one workflow-task profile.
///
/// Semantic field boundary: this public DTO preserves externally serialized
/// TOML field names.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QianjiWorkflowLlmEndpointConfig {
    /// Provider label used for operator/debug routing metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Optional model override for this workflow-task profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Optional OpenAI-compatible base URL override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// Environment variable name containing the provider API key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,
    /// OpenAI-compatible wire mode, such as `chat_completions` or `responses`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wire_api: Option<String>,
}

/// Activity-task route settings for model-backed workflow host work.
///
/// Semantic field boundary: this public DTO preserves externally serialized
/// TOML field names.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QianjiWorkflowLlmTaskRouteConfig {
    /// Control-plane activity type, typically `llm.plan`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) activity_type: Option<String>,
    /// Control-plane task queue, such as `llm.openrouter`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) task_queue: Option<String>,
    /// Prefix used when deriving deterministic idempotency keys.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) idempotency_key_prefix: Option<String>,
    /// Artifact kind required for the prompt claim-check artifact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) prompt_artifact_kind: Option<String>,
    /// Artifact kind used for optional context claim-check artifacts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) context_artifact_kind: Option<String>,
    /// Artifact kind used for optional response-schema claim-check artifacts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) response_schema_artifact_kind: Option<String>,
    /// Request temperature encoded in thousandths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) temperature_millis: Option<u32>,
    /// Maximum completion tokens for the provider request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) max_tokens: Option<u32>,
    /// Activity execution timeout in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) timeout_ms: Option<u64>,
    /// Optional retry policy for the admitted LLM activity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) retry: Option<QianjiWorkflowLlmTaskRetryConfig>,
}

/// Retry policy overlay for model-backed workflow activity tasks.
///
/// Semantic field boundary: this public DTO preserves externally serialized
/// TOML field names.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QianjiWorkflowLlmTaskRetryConfig {
    /// Maximum attempts including the first attempt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) max_attempts: Option<u32>,
    /// Initial retry interval in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) initial_interval_ms: Option<u64>,
    /// Maximum retry interval in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) max_interval_ms: Option<u64>,
    /// Backoff multiplier encoded in thousandths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) backoff_multiplier_millis: Option<u32>,
    /// Error codes that should not be retried.
    #[serde(default)]
    pub(crate) non_retryable_error_codes: Vec<String>,
}

impl QianjiWorkflowLlmTaskConfig {
    pub(crate) fn apply_overlay(&mut self, overlay: Self) {
        if let Some(schema) = non_empty(overlay.schema) {
            self.schema = Some(schema);
        }
        self.llm.apply_overlay(overlay.llm);
        self.task.apply_overlay(overlay.task);
    }
}

impl QianjiWorkflowLlmEndpointConfig {
    fn apply_overlay(&mut self, overlay: Self) {
        if let Some(provider) = non_empty(overlay.provider) {
            self.provider = Some(provider);
        }
        if let Some(model) = non_empty(overlay.model) {
            self.model = Some(model);
        }
        if let Some(base_url) = non_empty(overlay.base_url) {
            self.base_url = Some(base_url);
        }
        if let Some(api_key_env) = non_empty(overlay.api_key_env) {
            self.api_key_env = Some(api_key_env);
        }
        if let Some(wire_api) = non_empty(overlay.wire_api) {
            self.wire_api = Some(wire_api);
        }
    }
}

impl QianjiWorkflowLlmTaskRouteConfig {
    fn apply_overlay(&mut self, overlay: Self) {
        if let Some(activity_type) = non_empty(overlay.activity_type) {
            self.activity_type = Some(activity_type);
        }
        if let Some(task_queue) = non_empty(overlay.task_queue) {
            self.task_queue = Some(task_queue);
        }
        if let Some(idempotency_key_prefix) = non_empty(overlay.idempotency_key_prefix) {
            self.idempotency_key_prefix = Some(idempotency_key_prefix);
        }
        if let Some(prompt_artifact_kind) = non_empty(overlay.prompt_artifact_kind) {
            self.prompt_artifact_kind = Some(prompt_artifact_kind);
        }
        if let Some(context_artifact_kind) = non_empty(overlay.context_artifact_kind) {
            self.context_artifact_kind = Some(context_artifact_kind);
        }
        if let Some(response_schema_artifact_kind) =
            non_empty(overlay.response_schema_artifact_kind)
        {
            self.response_schema_artifact_kind = Some(response_schema_artifact_kind);
        }
        if let Some(temperature_millis) = overlay.temperature_millis {
            self.temperature_millis = Some(temperature_millis);
        }
        if let Some(max_tokens) = overlay.max_tokens {
            self.max_tokens = Some(max_tokens);
        }
        if let Some(timeout_ms) = overlay.timeout_ms {
            self.timeout_ms = Some(timeout_ms);
        }
        match (self.retry.as_mut(), overlay.retry) {
            (Some(target), Some(overlay)) => target.apply_overlay(overlay),
            (None, Some(overlay)) => self.retry = Some(overlay),
            _ => {}
        }
    }
}

impl QianjiWorkflowLlmTaskRetryConfig {
    fn apply_overlay(&mut self, overlay: Self) {
        if let Some(max_attempts) = overlay.max_attempts {
            self.max_attempts = Some(max_attempts);
        }
        if let Some(initial_interval_ms) = overlay.initial_interval_ms {
            self.initial_interval_ms = Some(initial_interval_ms);
        }
        if let Some(max_interval_ms) = overlay.max_interval_ms {
            self.max_interval_ms = Some(max_interval_ms);
        }
        if let Some(backoff_multiplier_millis) = overlay.backoff_multiplier_millis {
            self.backoff_multiplier_millis = Some(backoff_multiplier_millis);
        }
        if !overlay.non_retryable_error_codes.is_empty() {
            self.non_retryable_error_codes = overlay
                .non_retryable_error_codes
                .into_iter()
                .filter_map(|value| non_empty(Some(value)))
                .collect();
        }
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_owned())
        .filter(|raw| !raw.is_empty())
}
