//! Runtime projection helpers for linked builtin rerank policy.

use serde_yaml::Value;
use xiuxian_wendao_core::capabilities::PluginCapabilityBinding;
use xiuxian_wendao_runtime::transport::RerankScoreWeights;

use xiuxian_wendao_julia::compatibility::link_graph::LinkGraphJuliaRerankRuntimeConfig;

/// Generic rerank projection resolved from the linked builtin plugin bundle.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BuiltinRerankRuntimeProjection {
    /// Selected rerank provider binding, if one is configured.
    pub binding: Option<PluginCapabilityBinding>,
    /// Rerank-side schema version, if one is configured.
    pub schema_version: Option<String>,
    /// Shared rerank score weights, if any are configured.
    pub score_weights: Option<RerankScoreWeights>,
}

/// Resolve builtin rerank projection from merged Wendao settings.
#[must_use]
pub fn resolve_builtin_rerank_runtime_projection_with_settings(
    settings: &Value,
) -> BuiltinRerankRuntimeProjection {
    project_julia_rerank_runtime(&LinkGraphJuliaRerankRuntimeConfig::resolve_with_settings(
        settings,
    ))
}

fn project_julia_rerank_runtime(
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
) -> BuiltinRerankRuntimeProjection {
    BuiltinRerankRuntimeProjection {
        binding: runtime.rerank_provider_binding(),
        schema_version: runtime
            .schema_version
            .clone()
            .filter(|value| !value.trim().is_empty()),
        score_weights: build_score_weights(runtime),
    }
}

fn build_score_weights(runtime: &LinkGraphJuliaRerankRuntimeConfig) -> Option<RerankScoreWeights> {
    let defaults = RerankScoreWeights::default();
    let vector_weight = runtime.vector_weight;
    let similarity_weight = runtime.similarity_weight;

    if vector_weight.is_none() && similarity_weight.is_none() {
        return None;
    }

    RerankScoreWeights::new(
        vector_weight.unwrap_or(defaults.vector_weight),
        similarity_weight.unwrap_or(defaults.semantic_weight),
    )
    .ok()
}
