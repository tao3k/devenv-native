use axum::body::to_bytes;
use axum::extract::{Path, Query, State};
use chrono::DateTime;
use serial_test::serial;

use crate::gateway::studio::types::UiPluginArtifact;
use xiuxian_wendao_builtin::{
    linked_builtin_julia_gateway_artifact_base_url,
    linked_builtin_julia_gateway_artifact_expected_toml_fragments,
    linked_builtin_julia_gateway_artifact_route,
    linked_builtin_julia_gateway_artifact_runtime_config_toml,
    linked_builtin_julia_gateway_artifact_schema_version,
    linked_builtin_julia_gateway_artifact_selected_transport,
    linked_builtin_julia_gateway_launcher_path,
};

use super::support::plugin_artifact_state;

#[tokio::test]
#[serial]
async fn plugin_artifact_handler_returns_resolved_artifact() {
    let (state, artifact_path) =
        plugin_artifact_state(&linked_builtin_julia_gateway_artifact_runtime_config_toml());

    let response = crate::gateway::studio::router::handlers::capabilities::get_plugin_artifact(
        State(state),
        Path(artifact_path.clone()),
        Query(
            crate::gateway::studio::router::handlers::capabilities::PluginArtifactQuery {
                format: None,
            },
        ),
    )
    .await
    .unwrap_or_else(|error| panic!("deployment artifact handler should resolve: {error:?}"));

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_else(|error| panic!("read json body: {error}"));
    let artifact: UiPluginArtifact = serde_json::from_slice(&body)
        .unwrap_or_else(|error| panic!("decode artifact json: {error}"));

    assert_eq!(artifact.plugin_id, artifact_path.plugin_id);
    assert_eq!(artifact.artifact_id, artifact_path.artifact_id);
    assert_eq!(
        artifact.artifact_schema_version,
        linked_builtin_julia_gateway_artifact_schema_version()
    );
    DateTime::parse_from_rfc3339(&artifact.generated_at)
        .unwrap_or_else(|error| panic!("parse artifact generated_at: {error}"));
    assert_eq!(
        artifact.base_url.as_deref(),
        Some(linked_builtin_julia_gateway_artifact_base_url())
    );
    assert_eq!(
        artifact.route.as_deref(),
        Some(linked_builtin_julia_gateway_artifact_route())
    );
    assert_eq!(
        artifact.schema_version.as_deref(),
        Some(linked_builtin_julia_gateway_artifact_schema_version())
    );
    assert_eq!(
        artifact.selected_transport,
        Some(crate::gateway::studio::types::UiPluginTransportKind::ArrowFlight)
    );
    assert_eq!(artifact.fallback_from, None);
    assert_eq!(artifact.fallback_reason, None);
    assert_eq!(
        artifact
            .launch
            .as_ref()
            .map(|launch| launch.launcher_path.as_str()),
        Some(linked_builtin_julia_gateway_launcher_path())
    );
}

#[tokio::test]
#[serial]
async fn plugin_artifact_handler_returns_canonical_json_shape() {
    let (state, artifact_path) =
        plugin_artifact_state(&linked_builtin_julia_gateway_artifact_runtime_config_toml());

    let response = crate::gateway::studio::router::handlers::capabilities::get_plugin_artifact(
        State(state),
        Path(artifact_path.clone()),
        Query(
            crate::gateway::studio::router::handlers::capabilities::PluginArtifactQuery {
                format: None,
            },
        ),
    )
    .await
    .unwrap_or_else(|error| panic!("deployment artifact handler should resolve: {error:?}"));

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_else(|error| panic!("read json body: {error}"));
    let artifact: serde_json::Value = serde_json::from_slice(&body)
        .unwrap_or_else(|error| panic!("decode artifact json: {error}"));

    assert_eq!(artifact["pluginId"], artifact_path.plugin_id);
    assert_eq!(artifact["artifactId"], artifact_path.artifact_id);
    assert_eq!(
        artifact["selectedTransport"],
        linked_builtin_julia_gateway_artifact_selected_transport()
    );
    assert_eq!(
        artifact["launch"]["launcherPath"],
        linked_builtin_julia_gateway_launcher_path()
    );
}

#[tokio::test]
#[serial]
async fn plugin_artifact_handler_returns_toml_when_requested() {
    let (state, artifact_path) =
        plugin_artifact_state(&linked_builtin_julia_gateway_artifact_runtime_config_toml());

    let response = crate::gateway::studio::router::handlers::capabilities::get_plugin_artifact(
        State(state),
        Path(artifact_path),
        Query(
            crate::gateway::studio::router::handlers::capabilities::PluginArtifactQuery {
                format: Some(crate::zhenfa_router::native::WendaoPluginArtifactOutputFormat::Toml),
            },
        ),
    )
    .await
    .unwrap_or_else(|error| panic!("deployment artifact toml handler should resolve: {error:?}"));

    let content_type = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_else(|error| panic!("read toml body: {error}"));
    let body_text =
        String::from_utf8(body.to_vec()).unwrap_or_else(|error| panic!("utf8 toml body: {error}"));

    assert_eq!(content_type, "text/plain; charset=utf-8");
    for expected_fragment in linked_builtin_julia_gateway_artifact_expected_toml_fragments() {
        assert!(
            body_text.contains(&expected_fragment),
            "expected rendered TOML to contain `{expected_fragment}`"
        );
    }
}
