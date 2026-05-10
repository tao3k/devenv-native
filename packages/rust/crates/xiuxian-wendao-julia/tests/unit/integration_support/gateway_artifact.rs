use super::{
    DEFAULT_JULIA_SEARCH_LAUNCHER_PATH, julia_gateway_artifact_expected_json_fragments,
    julia_gateway_artifact_expected_toml_fragments, julia_gateway_artifact_path,
    julia_gateway_artifact_rpc_params_fixture, julia_gateway_artifact_runtime_config_toml,
    julia_gateway_artifact_selected_transport, julia_ui_artifact_payload_fixture,
};
use xiuxian_wendao_core::transport::PluginTransportKind;

#[test]
fn gateway_artifact_runtime_config_toml_renders_default_fixture() {
    let rendered = julia_gateway_artifact_runtime_config_toml();

    assert!(rendered.contains("[link_graph.retrieval.julia_rerank]"));
    assert!(rendered.contains("base_url = \"http://127.0.0.1:18080\""));
    assert!(!rendered.contains("analyzer"));
}

#[test]
fn gateway_artifact_helpers_keep_stable_selector_and_toml_fragments() {
    let artifact_path = julia_gateway_artifact_path();
    let fragments = julia_gateway_artifact_expected_toml_fragments();

    assert_eq!(artifact_path.plugin_id, "xiuxian-wendao-julia");
    assert_eq!(artifact_path.artifact_id, "deployment");
    assert!(
        fragments
            .iter()
            .any(|fragment| fragment.contains(julia_gateway_artifact_selected_transport()))
    );
}

#[test]
fn gateway_artifact_helpers_keep_stable_json_fragments() {
    let fragments = julia_gateway_artifact_expected_json_fragments();

    assert!(
        fragments
            .iter()
            .any(|fragment| fragment.contains("\"artifact_schema_version\": \"v1\""))
    );
    assert!(
        fragments
            .iter()
            .any(|fragment| fragment.contains("\"route\": \"/rerank\""))
    );
}

#[test]
fn gateway_artifact_rpc_params_fixture_keeps_stable_shape() {
    let request = julia_gateway_artifact_rpc_params_fixture(
        Some("json"),
        Some(".run/julia/plugin-artifact.json"),
    );

    assert_eq!(request["plugin_id"].as_str(), Some("xiuxian-wendao-julia"));
    assert_eq!(request["artifact_id"].as_str(), Some("deployment"));
    assert_eq!(request["output_format"].as_str(), Some("json"));
    assert_eq!(
        request["output_path"].as_str(),
        Some(".run/julia/plugin-artifact.json")
    );
}

#[test]
fn ui_artifact_payload_fixture_keeps_stable_julia_payload_shape() {
    let payload = julia_ui_artifact_payload_fixture();
    let endpoint = payload
        .endpoint
        .as_ref()
        .unwrap_or_else(|| panic!("fixture should include endpoint"));
    let launch = payload
        .launch
        .as_ref()
        .unwrap_or_else(|| panic!("fixture should include launch spec"));

    assert_eq!(payload.plugin_id.0, "xiuxian-wendao-julia");
    assert_eq!(payload.artifact_id.0, "deployment");
    assert_eq!(payload.generated_at, "2026-03-27T12:00:00Z");
    assert_eq!(endpoint.base_url.as_deref(), Some("http://127.0.0.1:8088"));
    assert_eq!(endpoint.health_route.as_deref(), Some("/healthz"));
    assert_eq!(endpoint.timeout_secs, Some(15));
    assert_eq!(
        payload.selected_transport,
        Some(PluginTransportKind::ArrowFlight)
    );
    assert_eq!(
        launch.launcher_path,
        DEFAULT_JULIA_SEARCH_LAUNCHER_PATH.to_string()
    );
}
