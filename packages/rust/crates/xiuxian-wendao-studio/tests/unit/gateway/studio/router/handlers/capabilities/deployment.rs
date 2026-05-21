use std::fs;
use std::sync::Arc;

use axum::body::to_bytes;
use axum::extract::{Path, Query, State};
use serial_test::serial;
use xiuxian_wendao_builtin::{
    linked_builtin_julia_gateway_artifact_path,
    linked_builtin_julia_gateway_artifact_runtime_config_toml,
    linked_builtin_julia_gateway_artifact_schema_version,
};

use crate::contracts::UiPluginArtifact;
use crate::studio::router::handlers::capabilities::deployment::get_plugin_artifact;
use crate::studio::router::handlers::capabilities::types::{
    PluginArtifactPath, PluginArtifactQuery,
};
use crate::studio::router::{GatewayState, StudioState};
use xiuxian_wendao::set_link_graph_wendao_config_override;
use xiuxian_wendao::zhenfa_router::native::WendaoPluginArtifactOutputFormat;

#[tokio::test]
#[serial]
async fn generic_plugin_artifact_handler_returns_plugin_artifact() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let config_path = temp.path().join("wendao.toml");
    let artifact_path = linked_builtin_julia_gateway_artifact_path();
    let plugin_id = artifact_path.plugin_id;
    let artifact_id = artifact_path.artifact_id;
    fs::write(
        &config_path,
        linked_builtin_julia_gateway_artifact_runtime_config_toml(),
    )
    .unwrap_or_else(|error| panic!("write config: {error}"));
    let config_path_string = config_path.to_string_lossy().to_string();
    set_link_graph_wendao_config_override(&config_path_string);

    let state = Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(StudioState::new()),
    });

    let response = get_plugin_artifact(
        State(Arc::clone(&state)),
        Path(PluginArtifactPath {
            plugin_id: plugin_id.clone(),
            artifact_id: artifact_id.clone(),
        }),
        Query(PluginArtifactQuery {
            format: Some(WendaoPluginArtifactOutputFormat::Json),
        }),
    )
    .await
    .unwrap_or_else(|error| {
        panic!("generic deployment artifact handler should resolve: {error:?}")
    });

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_else(|error| panic!("read json body: {error}"));
    let artifact: UiPluginArtifact = serde_json::from_slice(&body)
        .unwrap_or_else(|error| panic!("decode artifact json: {error}"));

    assert_eq!(artifact.plugin_id, plugin_id);
    assert_eq!(artifact.artifact_id, artifact_id);
    assert_eq!(
        artifact.schema_version.as_deref(),
        Some(linked_builtin_julia_gateway_artifact_schema_version())
    );
    assert_eq!(artifact.route.as_deref(), Some("/rerank"));
}
