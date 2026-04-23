use xiuxian_testing::AdvisoryAuditRequest;
use xiuxian_zhenfa::{CognitiveDistribution, StreamProvider};

use super::DEFAULT_MODEL;

#[derive(Debug, Clone)]
pub(super) struct LiveCognitiveMetrics {
    pub(super) coherence: f32,
    pub(super) early_halt: Option<String>,
    pub(super) distribution: CognitiveDistribution,
}

pub(super) fn resolve_model(request: &AdvisoryAuditRequest, default_model: &str) -> String {
    request
        .collection_context
        .labels
        .get("llm_model")
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .or_else(|| {
            let trimmed = default_model.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .unwrap_or_else(|| DEFAULT_MODEL.to_string())
}

pub(super) fn resolve_provider(model: &str) -> StreamProvider {
    let model_lower = model.to_ascii_lowercase();
    if model_lower.contains("claude") || model_lower.contains("anthropic") {
        StreamProvider::Claude
    } else if model_lower.contains("gemini") {
        StreamProvider::Gemini
    } else {
        StreamProvider::Codex
    }
}
