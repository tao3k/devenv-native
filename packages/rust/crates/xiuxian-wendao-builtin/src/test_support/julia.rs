//! Julia selector and binding fixtures for builtin integration tests.

use xiuxian_julia_runtime::wendao::link_graph::{
    DEFAULT_JULIA_SEARCH_EXAMPLE_CONFIG_PATH, DEFAULT_JULIA_SEARCH_LAUNCHER_PATH,
    LinkGraphJuliaRerankRuntimeConfig, build_rerank_provider_binding,
    julia_deployment_artifact_selector, julia_rerank_provider_selector,
};
use xiuxian_wendao_core::{
    PluginCapabilityBinding, PluginProviderSelector, artifacts::PluginArtifactSelector,
};

/// Return the linked builtin Julia search example config path.
#[must_use]
pub fn linked_builtin_julia_search_example_config_path() -> &'static str {
    DEFAULT_JULIA_SEARCH_EXAMPLE_CONFIG_PATH
}

/// Return the linked builtin Julia search launcher path.
#[must_use]
pub fn linked_builtin_julia_search_launcher_path() -> &'static str {
    DEFAULT_JULIA_SEARCH_LAUNCHER_PATH
}

/// Return the linked builtin Julia rerank provider selector.
#[must_use]
pub fn linked_builtin_julia_rerank_provider_selector() -> PluginProviderSelector {
    julia_rerank_provider_selector()
}

/// Return the linked builtin Julia deployment artifact selector.
#[must_use]
pub fn linked_builtin_julia_deployment_artifact_selector() -> PluginArtifactSelector {
    julia_deployment_artifact_selector()
}

/// Build a linked builtin Julia rerank binding from endpoint overrides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedBuiltinJuliaRerankEndpoint {
    /// Base URL for the Julia rerank service.
    pub base_url: String,
    /// Flight route used for rerank requests.
    pub route: String,
    /// Health route used for readiness checks.
    pub health_route: String,
    /// Expected schema version for rerank payloads.
    pub schema_version: String,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
}

/// Build a linked builtin Julia rerank binding from endpoint overrides.
#[must_use]
pub fn linked_builtin_julia_rerank_provider_binding_with_endpoint(
    endpoint: &LinkedBuiltinJuliaRerankEndpoint,
) -> PluginCapabilityBinding {
    build_rerank_provider_binding(&LinkGraphJuliaRerankRuntimeConfig {
        base_url: Some(endpoint.base_url.clone().into()),
        route: Some(endpoint.route.clone().into()),
        health_route: Some(endpoint.health_route.clone().into()),
        schema_version: Some(endpoint.schema_version.clone().into()),
        timeout_secs: Some(endpoint.timeout_secs.into()),
        service_mode: None,
        search_config_path: None,
        vector_weight: None,
        similarity_weight: None,
    })
}
