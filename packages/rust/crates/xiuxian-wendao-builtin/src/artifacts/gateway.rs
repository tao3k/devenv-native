//! Gateway artifact fixtures for linked builtin Julia services.

use serde_json::Value;
use xiuxian_wendao_core::artifacts::PluginArtifactPayload;
use xiuxian_wendao_julia::compatibility::link_graph::{
    DEFAULT_JULIA_RERANK_FLIGHT_ROUTE, DEFAULT_JULIA_SEARCH_LAUNCHER_PATH,
};
use xiuxian_wendao_julia::integration_support::{
    julia_gateway_artifact_base_url, julia_gateway_artifact_expected_json_fragments,
    julia_gateway_artifact_expected_toml_fragments, julia_gateway_artifact_path,
    julia_gateway_artifact_rpc_params_fixture, julia_gateway_artifact_runtime_config_toml,
    julia_gateway_artifact_schema_version, julia_gateway_artifact_selected_transport,
    julia_ui_artifact_payload_fixture,
};

/// Return the linked builtin Julia gateway artifact base URL fixture.
#[must_use]
pub fn linked_builtin_julia_gateway_artifact_base_url() -> &'static str {
    julia_gateway_artifact_base_url()
}

/// Return the linked builtin Julia gateway artifact schema version fixture.
#[must_use]
pub fn linked_builtin_julia_gateway_artifact_schema_version() -> &'static str {
    julia_gateway_artifact_schema_version()
}

/// Return the linked builtin Julia gateway artifact selected-transport fixture.
#[must_use]
pub fn linked_builtin_julia_gateway_artifact_selected_transport() -> &'static str {
    julia_gateway_artifact_selected_transport()
}

/// Return the linked builtin Julia gateway artifact selector fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedBuiltinJuliaGatewayArtifactPath {
    /// Plugin identifier that owns the gateway artifact.
    pub plugin_id: String,
    /// Artifact identifier inside the plugin.
    pub artifact_id: String,
}

impl LinkedBuiltinJuliaGatewayArtifactPath {
    /// Builds a named artifact path from the upstream Julia tuple fixture.
    #[must_use]
    pub fn from_tuple((plugin_id, artifact_id): (String, String)) -> Self {
        Self {
            plugin_id,
            artifact_id,
        }
    }
}

/// Return the linked builtin Julia gateway artifact selector fixture.
#[must_use]
pub fn linked_builtin_julia_gateway_artifact_path() -> LinkedBuiltinJuliaGatewayArtifactPath {
    LinkedBuiltinJuliaGatewayArtifactPath::from_tuple(julia_gateway_artifact_path())
}

/// Return the linked builtin Julia gateway artifact route fixture.
#[must_use]
pub fn linked_builtin_julia_gateway_artifact_route() -> &'static str {
    DEFAULT_JULIA_RERANK_FLIGHT_ROUTE
}

/// Return the linked builtin Julia gateway launcher-path fixture.
#[must_use]
pub fn linked_builtin_julia_gateway_launcher_path() -> &'static str {
    DEFAULT_JULIA_SEARCH_LAUNCHER_PATH
}

/// Render the linked builtin Julia gateway runtime-config TOML fixture.
#[must_use]
pub fn linked_builtin_julia_gateway_artifact_runtime_config_toml() -> String {
    julia_gateway_artifact_runtime_config_toml()
}

/// Return the linked builtin Julia gateway expected TOML fragments.
#[must_use]
pub fn linked_builtin_julia_gateway_artifact_expected_toml_fragments() -> Vec<String> {
    julia_gateway_artifact_expected_toml_fragments()
}

/// Return the linked builtin Julia gateway expected JSON fragments.
#[must_use]
pub fn linked_builtin_julia_gateway_artifact_expected_json_fragments() -> Vec<String> {
    julia_gateway_artifact_expected_json_fragments()
}

/// Build the linked builtin Julia gateway JSON-RPC params fixture.
#[must_use]
pub fn linked_builtin_julia_gateway_artifact_rpc_params_fixture(
    output_format: Option<&str>,
    output_path: Option<&str>,
) -> Value {
    julia_gateway_artifact_rpc_params_fixture(output_format, output_path)
}

/// Return the linked builtin Julia gateway UI artifact payload fixture.
#[must_use]
pub fn linked_builtin_julia_gateway_artifact_ui_payload_fixture() -> PluginArtifactPayload {
    julia_ui_artifact_payload_fixture()
}
