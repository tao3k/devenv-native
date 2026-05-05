use crate::contracts::{UiPluginArtifact, UiPluginLaunchSpec};
use xiuxian_wendao_builtin::linked_builtin_julia_gateway_artifact_ui_payload_fixture;

#[test]
fn generic_ui_artifact_builds_from_plugin_artifact_payload() {
    let payload = linked_builtin_julia_gateway_artifact_ui_payload_fixture();
    let expected_endpoint = payload
        .endpoint
        .clone()
        .unwrap_or_else(|| panic!("fixture should include endpoint"));
    let expected_launch = payload
        .launch
        .clone()
        .unwrap_or_else(|| panic!("fixture should include launch spec"));
    let expected_plugin_id = payload.plugin_id.0.clone();
    let expected_artifact_id = payload.artifact_id.0.clone();
    let expected_artifact_schema_version = payload.artifact_schema_version.0.clone();
    let expected_generated_at = payload.generated_at.clone();
    let expected_schema_version = payload.schema_version.clone();
    let expected_selected_transport = payload.selected_transport;
    let expected_fallback_from = payload.fallback_from;
    let expected_fallback_reason = payload.fallback_reason.clone();

    assert_eq!(
        UiPluginArtifact::from(payload),
        UiPluginArtifact {
            plugin_id: expected_plugin_id,
            artifact_id: expected_artifact_id,
            artifact_schema_version: expected_artifact_schema_version,
            generated_at: expected_generated_at,
            base_url: expected_endpoint.base_url,
            route: expected_endpoint.route,
            health_route: expected_endpoint.health_route,
            timeout_secs: expected_endpoint.timeout_secs,
            schema_version: expected_schema_version,
            launch: Some(UiPluginLaunchSpec {
                launcher_path: expected_launch.launcher_path,
                args: expected_launch.args,
            }),
            selected_transport: expected_selected_transport.map(Into::into),
            fallback_from: expected_fallback_from.map(Into::into),
            fallback_reason: expected_fallback_reason,
        }
    );
}
