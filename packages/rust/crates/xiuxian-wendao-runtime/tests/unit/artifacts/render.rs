use super::{render_plugin_artifact_toml_for_selector_with, render_plugin_artifact_toml_with};
use xiuxian_wendao_core::{
    artifacts::{PluginArtifactPayload, PluginArtifactSelector},
    capabilities::ContractVersion,
    ids::{ArtifactId, PluginId},
};

fn sample_payload() -> PluginArtifactPayload {
    PluginArtifactPayload {
        plugin_id: PluginId("xiuxian-julia-core".to_string()),
        artifact_id: ArtifactId("deployment".to_string()),
        artifact_schema_version: ContractVersion("v1".to_string()),
        generated_at: "2026-03-28T12:00:00Z".to_string(),
        endpoint: None,
        schema_version: Some("v1".to_string()),
        launch: None,
        selected_transport: None,
        fallback_from: None,
        fallback_reason: None,
    }
}

#[test]
fn render_plugin_artifact_toml_with_serializes_resolved_payload() {
    let rendered = render_plugin_artifact_toml_with(
        "xiuxian-julia-core",
        "deployment",
        |_plugin_id, _artifact_id| Some(sample_payload()),
    )
    .unwrap_or_else(|error| panic!("render should succeed: {error}"))
    .unwrap_or_else(|| panic!("rendered payload should exist"));

    assert!(rendered.contains("plugin_id = \"xiuxian-julia-core\""));
    assert!(rendered.contains("artifact_id = \"deployment\""));
}

#[test]
fn render_plugin_artifact_toml_for_selector_with_serializes_resolved_payload() {
    let selector = PluginArtifactSelector {
        plugin_id: PluginId("xiuxian-julia-core".to_string()),
        artifact_id: ArtifactId("deployment".to_string()),
    };

    let rendered =
        render_plugin_artifact_toml_for_selector_with(
            &selector,
            |_selector| Some(sample_payload()),
        )
        .unwrap_or_else(|error| panic!("render should succeed: {error}"))
        .unwrap_or_else(|| panic!("rendered payload should exist"));

    assert!(rendered.contains("artifact_schema_version = \"v1\""));
    assert!(rendered.contains("schema_version = \"v1\""));
}
