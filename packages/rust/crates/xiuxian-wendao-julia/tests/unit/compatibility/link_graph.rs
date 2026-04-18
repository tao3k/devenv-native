use serde_yaml::Value;
use std::fs;
use xiuxian_wendao_core::{
    artifacts::{PluginArtifactPayload, PluginLaunchSpec},
    transport::PluginTransportEndpoint,
};

use super::{
    DEFAULT_JULIA_ANALYZER_EXAMPLE_CONFIG_PATH, DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH,
    DEFAULT_JULIA_DEPLOYMENT_ARTIFACT_SCHEMA_VERSION, DEFAULT_JULIA_RERANK_FLIGHT_ROUTE,
    JULIA_DEPLOYMENT_ARTIFACT_ID, JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID, JULIA_PLUGIN_ID,
    JULIA_RERANK_CAPABILITY_ID, LINK_GRAPH_JULIA_RERANK_ANALYZER_CONFIG_PATH_ENV,
    LINK_GRAPH_JULIA_RERANK_ANALYZER_STRATEGY_ENV, LINK_GRAPH_JULIA_RERANK_BASE_URL_ENV,
    LINK_GRAPH_JULIA_RERANK_HEALTH_ROUTE_ENV, LINK_GRAPH_JULIA_RERANK_ROUTE_ENV,
    LINK_GRAPH_JULIA_RERANK_SCHEMA_VERSION_ENV, LINK_GRAPH_JULIA_RERANK_SERVICE_MODE_ENV,
    LINK_GRAPH_JULIA_RERANK_SIMILARITY_WEIGHT_ENV, LINK_GRAPH_JULIA_RERANK_TIMEOUT_SECS_ENV,
    LINK_GRAPH_JULIA_RERANK_VECTOR_WEIGHT_ENV, LinkGraphJuliaAnalyzerLaunchManifest,
    LinkGraphJuliaAnalyzerServiceDescriptor, LinkGraphJuliaDeploymentArtifact,
    LinkGraphJuliaRerankRuntimeConfig, build_rerank_provider_binding,
    julia_deployment_artifact_openapi_example, julia_deployment_artifact_openapi_json_example,
    julia_deployment_artifact_openapi_toml_example, julia_deployment_artifact_selector,
    julia_graph_structural_provider_selector, julia_plugin_artifact_openapi_json_example,
    julia_plugin_artifact_openapi_toml_example, julia_rerank_provider_selector,
    render_julia_plugin_artifact_toml_for_selector,
    resolve_julia_plugin_artifact_payload_for_selector,
};

include!("link_graph/selectors_and_launch.rs");
include!("link_graph/artifact_contract.rs");
include!("link_graph/runtime_config.rs");
include!("link_graph/plugin_artifact.rs");
