use super::{resolve_plugin_artifact_for_selector_with, resolve_plugin_artifact_with};
use xiuxian_wendao_core::{
    artifacts::{PluginArtifactPayload, PluginArtifactSelector},
    capabilities::ContractVersion,
    ids::{ArtifactId, PluginId},
};

fn sample_selector() -> PluginArtifactSelector {
    PluginArtifactSelector {
        plugin_id: PluginId("xiuxian-julia-core".to_string()),
        artifact_id: ArtifactId("deployment".to_string()),
    }
}

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
fn resolve_plugin_artifact_with_builds_selector_before_delegating() {
    let artifact = resolve_plugin_artifact_with("xiuxian-julia-core", "deployment", |selector| {
        assert_eq!(selector, &sample_selector());
        Some(sample_payload())
    })
    .unwrap_or_else(|| panic!("artifact should resolve"));

    assert_eq!(artifact.plugin_id.0, "xiuxian-julia-core");
    assert_eq!(artifact.artifact_id.0, "deployment");
}

#[test]
fn resolve_plugin_artifact_for_selector_with_delegates_directly() {
    let selector = sample_selector();
    let artifact = resolve_plugin_artifact_for_selector_with(&selector, |observed| {
        assert_eq!(observed, &selector);
        Some(sample_payload())
    })
    .unwrap_or_else(|| panic!("artifact should resolve"));

    assert_eq!(artifact.artifact_schema_version.0, "v1");
}
