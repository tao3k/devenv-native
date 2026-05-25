//! Compatibility contracts for Julia-backed link-graph rerank and gateway artifacts.

mod artifact;
mod deployment;
mod launch;
mod openapi_examples;
mod paths;
mod runtime;
mod selectors;
mod settings;

pub use artifact::{
    render_julia_plugin_artifact_toml_for_selector, resolve_julia_deployment_artifact_payload,
    resolve_julia_plugin_artifact_payload_for_selector,
};
pub use deployment::{
    DEFAULT_JULIA_DEPLOYMENT_ARTIFACT_SCHEMA_VERSION, LinkGraphJuliaDeploymentArtifact,
};
pub use launch::{LinkGraphJuliaSearchLaunchManifest, LinkGraphJuliaSearchServiceDescriptor};
pub use openapi_examples::{
    julia_deployment_artifact_openapi_example, julia_deployment_artifact_openapi_json_example,
    julia_deployment_artifact_openapi_toml_example, julia_plugin_artifact_openapi_json_example,
    julia_plugin_artifact_openapi_toml_example,
};
pub use paths::{
    DEFAULT_JULIA_RERANK_FLIGHT_ROUTE, DEFAULT_JULIA_SEARCH_EXAMPLE_CONFIG_PATH,
    DEFAULT_JULIA_SEARCH_LAUNCHER_PATH,
};
pub use runtime::{LinkGraphJuliaRerankRuntimeConfig, build_rerank_provider_binding};
pub use selectors::{
    JULIA_CAPABILITY_MANIFEST_CAPABILITY_ID, JULIA_DEPLOYMENT_ARTIFACT_ID,
    JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID, JULIA_PARSER_SUMMARY_CAPABILITY_ID, JULIA_PLUGIN_ID,
    JULIA_RERANK_CAPABILITY_ID, julia_capability_manifest_provider_selector,
    julia_deployment_artifact_selector, julia_graph_structural_provider_selector,
    julia_parser_summary_provider_selector, julia_rerank_provider_selector,
};
pub use settings::{
    LINK_GRAPH_JULIA_RERANK_BASE_URL_ENV, LINK_GRAPH_JULIA_RERANK_HEALTH_ROUTE_ENV,
    LINK_GRAPH_JULIA_RERANK_ROUTE_ENV, LINK_GRAPH_JULIA_RERANK_SCHEMA_VERSION_ENV,
    LINK_GRAPH_JULIA_RERANK_SEARCH_CONFIG_PATH_ENV, LINK_GRAPH_JULIA_RERANK_SERVICE_MODE_ENV,
    LINK_GRAPH_JULIA_RERANK_SIMILARITY_WEIGHT_ENV, LINK_GRAPH_JULIA_RERANK_TIMEOUT_SECS_ENV,
    LINK_GRAPH_JULIA_RERANK_VECTOR_WEIGHT_ENV,
};
