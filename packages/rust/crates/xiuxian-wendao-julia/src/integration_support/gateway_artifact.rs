//! Fixture payloads for Julia gateway artifact integration checks.

use crate::compatibility::link_graph::{
    DEFAULT_JULIA_RERANK_FLIGHT_ROUTE, DEFAULT_JULIA_SEARCH_LAUNCHER_PATH,
    JULIA_DEPLOYMENT_ARTIFACT_ID, JULIA_PLUGIN_ID,
};
use serde_json::{Value, json};
use xiuxian_wendao_core::artifacts::PluginLaunchSpec;
use xiuxian_wendao_core::{
    artifacts::PluginArtifactPayload,
    capabilities::ContractVersion,
    ids::{ArtifactId, PluginId},
    transport::{PluginTransportEndpoint, PluginTransportKind},
};

const JULIA_GATEWAY_ARTIFACT_BASE_URL: &str = "http://127.0.0.1:18080";
const JULIA_GATEWAY_ARTIFACT_SCHEMA_VERSION: &str = "v1";
const JULIA_GATEWAY_ARTIFACT_SERVICE_MODE: &str = "stream";
const JULIA_GATEWAY_ARTIFACT_SELECTED_TRANSPORT: &str = "arrow_flight";
const JULIA_UI_ARTIFACT_BASE_URL: &str = "http://127.0.0.1:8088";
const JULIA_UI_ARTIFACT_GENERATED_AT: &str = "2026-03-27T12:00:00Z";
const JULIA_UI_ARTIFACT_HEALTH_ROUTE: &str = "/healthz";
const JULIA_UI_ARTIFACT_TIMEOUT_SECS: u64 = 15;

/// Return the stable base URL used by Studio gateway artifact fixtures.
#[must_use]
pub fn julia_gateway_artifact_base_url() -> &'static str {
    JULIA_GATEWAY_ARTIFACT_BASE_URL
}

/// Return the stable schema version used by Studio gateway artifact fixtures.
#[must_use]
pub fn julia_gateway_artifact_schema_version() -> &'static str {
    JULIA_GATEWAY_ARTIFACT_SCHEMA_VERSION
}

/// Return the stable selected transport used by Studio gateway artifact
/// fixtures.
#[must_use]
pub fn julia_gateway_artifact_selected_transport() -> &'static str {
    JULIA_GATEWAY_ARTIFACT_SELECTED_TRANSPORT
}

/// Return the stable path identifiers for the Julia deployment artifact.
#[must_use]
pub fn julia_gateway_artifact_path() -> (String, String) {
    (
        JULIA_PLUGIN_ID.to_string(),
        JULIA_DEPLOYMENT_ARTIFACT_ID.to_string(),
    )
}

/// Build the JSON-RPC params fixture used by generic Julia plugin-artifact
/// export tests.
#[must_use]
pub fn julia_gateway_artifact_rpc_params_fixture(
    output_format: Option<&str>,
    output_path: Option<&str>,
) -> Value {
    let (plugin_id, artifact_id) = julia_gateway_artifact_path();
    let mut request = json!({
        "plugin_id": plugin_id,
        "artifact_id": artifact_id,
    });

    if let Some(format) = output_format {
        request["output_format"] = Value::String(format.to_string());
    }

    if let Some(path) = output_path {
        request["output_path"] = Value::String(path.to_string());
    }

    request
}

/// Render the runtime config TOML fixture used by Studio gateway artifact
/// handler tests.
#[must_use]
pub fn julia_gateway_artifact_runtime_config_toml() -> String {
    format!(
        "[link_graph.retrieval.julia_rerank]\nbase_url = \"{JULIA_GATEWAY_ARTIFACT_BASE_URL}\"\nroute = \"{DEFAULT_JULIA_RERANK_FLIGHT_ROUTE}\"\nschema_version = \"{JULIA_GATEWAY_ARTIFACT_SCHEMA_VERSION}\"\nservice_mode = \"{JULIA_GATEWAY_ARTIFACT_SERVICE_MODE}\"\n"
    )
}

/// Return the stable TOML fragments that Studio gateway artifact handler tests
/// should observe in the rendered generic plugin artifact output.
#[must_use]
pub fn julia_gateway_artifact_expected_toml_fragments() -> Vec<String> {
    vec![
        format!("artifact_schema_version = \"{JULIA_GATEWAY_ARTIFACT_SCHEMA_VERSION}\""),
        format!("base_url = \"{JULIA_GATEWAY_ARTIFACT_BASE_URL}\""),
        format!("route = \"{DEFAULT_JULIA_RERANK_FLIGHT_ROUTE}\""),
        format!("launcher_path = \"{DEFAULT_JULIA_SEARCH_LAUNCHER_PATH}\""),
        format!("selected_transport = \"{JULIA_GATEWAY_ARTIFACT_SELECTED_TRANSPORT}\""),
    ]
}

/// Return the stable JSON fragments that generic plugin-artifact tests should
/// observe in rendered Julia deployment artifact output.
#[must_use]
pub fn julia_gateway_artifact_expected_json_fragments() -> Vec<String> {
    vec![
        format!("\"artifact_schema_version\": \"{JULIA_GATEWAY_ARTIFACT_SCHEMA_VERSION}\""),
        format!("\"base_url\": \"{JULIA_GATEWAY_ARTIFACT_BASE_URL}\""),
        format!("\"route\": \"{DEFAULT_JULIA_RERANK_FLIGHT_ROUTE}\""),
        format!("\"launcher_path\": \"{DEFAULT_JULIA_SEARCH_LAUNCHER_PATH}\""),
    ]
}

/// Build the Julia `PluginArtifactPayload` fixture used by Studio UI artifact
/// mapping tests.
#[must_use]
pub fn julia_ui_artifact_payload_fixture() -> PluginArtifactPayload {
    let (plugin_id, artifact_id) = julia_gateway_artifact_path();

    PluginArtifactPayload {
        plugin_id: PluginId(plugin_id),
        artifact_id: ArtifactId(artifact_id),
        artifact_schema_version: ContractVersion(JULIA_GATEWAY_ARTIFACT_SCHEMA_VERSION.to_string()),
        generated_at: JULIA_UI_ARTIFACT_GENERATED_AT.to_string(),
        endpoint: Some(PluginTransportEndpoint {
            base_url: Some(JULIA_UI_ARTIFACT_BASE_URL.to_string()),
            route: Some(DEFAULT_JULIA_RERANK_FLIGHT_ROUTE.to_string()),
            health_route: Some(JULIA_UI_ARTIFACT_HEALTH_ROUTE.to_string()),
            timeout_secs: Some(JULIA_UI_ARTIFACT_TIMEOUT_SECS),
            max_in_flight_requests: None,
        }),
        schema_version: Some(JULIA_GATEWAY_ARTIFACT_SCHEMA_VERSION.to_string()),
        launch: Some(PluginLaunchSpec {
            launcher_path: DEFAULT_JULIA_SEARCH_LAUNCHER_PATH.to_string(),
            args: vec![
                "--mode".to_string(),
                JULIA_GATEWAY_ARTIFACT_SERVICE_MODE.to_string(),
            ],
        }),
        selected_transport: Some(PluginTransportKind::ArrowFlight),
        fallback_from: None,
        fallback_reason: None,
    }
}

#[cfg(test)]
#[path = "../../tests/unit/integration_support/gateway_artifact.rs"]
mod tests;
