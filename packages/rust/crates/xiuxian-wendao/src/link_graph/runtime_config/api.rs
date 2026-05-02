//! Public runtime-config API helpers.

use super::resolve::resolve_link_graph_retrieval_policy_runtime;
use xiuxian_wendao_core::capabilities::PluginCapabilityBinding;
use xiuxian_wendao_runtime::transport::RerankScoreWeights;

/// File-backed runtime settings that can influence the Flight rerank host.
#[derive(Clone, Debug, PartialEq)]
pub struct LinkGraphRerankFlightRuntimeSettings {
    /// Schema version from retrieval policy config, if configured.
    pub schema_version: Option<String>,
    /// Score weights from retrieval policy config, if configured.
    pub score_weights: Option<RerankScoreWeights>,
}

/// Resolve the current retrieval rerank binding through the generic plugin-runtime model.
#[must_use]
pub fn resolve_link_graph_rerank_binding() -> Option<PluginCapabilityBinding> {
    resolve_link_graph_retrieval_policy_runtime().rerank_binding()
}

/// Resolve the current runtime-owned rerank score weights from Wendao
/// retrieval policy settings.
#[must_use]
pub fn resolve_link_graph_rerank_score_weights() -> Option<RerankScoreWeights> {
    resolve_link_graph_retrieval_policy_runtime().rerank_score_weights()
}

/// Resolve the current rerank-side schema version from Wendao retrieval
/// policy settings.
#[must_use]
pub fn resolve_link_graph_rerank_schema_version() -> Option<String> {
    resolve_link_graph_retrieval_policy_runtime().rerank_schema_version()
}

/// Resolve the current file-backed Flight rerank host settings from Wendao
/// retrieval policy configuration.
#[must_use]
pub fn resolve_link_graph_rerank_flight_runtime_settings() -> LinkGraphRerankFlightRuntimeSettings {
    LinkGraphRerankFlightRuntimeSettings {
        schema_version: resolve_link_graph_rerank_schema_version(),
        score_weights: resolve_link_graph_rerank_score_weights(),
    }
}
