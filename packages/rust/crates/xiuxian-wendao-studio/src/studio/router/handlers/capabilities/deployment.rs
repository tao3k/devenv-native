use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};

use crate::studio::router::{GatewayState, StudioApiError};
use crate::studio::types::UiPluginArtifact;
use xiuxian_wendao::link_graph::runtime_config::{
    render_link_graph_plugin_artifact_toml_for_selector,
    resolve_link_graph_plugin_artifact_for_selector,
};
use xiuxian_wendao::zhenfa_router::native::WendaoPluginArtifactOutputFormat;
use xiuxian_wendao_core::artifacts::PluginArtifactSelector;

use crate::studio::router::handlers::capabilities::types::{
    PluginArtifactPath, PluginArtifactQuery,
};

fn render_plugin_artifact_json_response(
    selector: &PluginArtifactSelector,
) -> Result<Response, StudioApiError> {
    let artifact = resolve_link_graph_plugin_artifact_for_selector(selector).ok_or_else(|| {
        StudioApiError::internal(
            "PLUGIN_ARTIFACT_RESOLVE_FAILED",
            "Failed to resolve plugin artifact",
            None,
        )
    })?;

    Ok(Json(UiPluginArtifact::from(artifact)).into_response())
}

fn render_plugin_artifact_toml_response(
    selector: &PluginArtifactSelector,
) -> Result<Response, StudioApiError> {
    let body = render_link_graph_plugin_artifact_toml_for_selector(selector)
        .map_err(|error| {
            StudioApiError::internal(
                "PLUGIN_ARTIFACT_EXPORT_FAILED",
                "Failed to render plugin artifact as TOML",
                Some(error.to_string()),
            )
        })?
        .ok_or_else(|| {
            StudioApiError::internal(
                "PLUGIN_ARTIFACT_EXPORT_FAILED",
                "Failed to render plugin artifact as TOML",
                None,
            )
        })?;

    Ok((
        StatusCode::OK,
        [(
            CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        )],
        body,
    )
        .into_response())
}

/// Read the currently resolved generic plugin artifact used by runtime config.
///
/// # Errors
///
/// This handler currently does not produce handler-local errors.
pub async fn get_plugin_artifact(
    State(_state): State<Arc<GatewayState>>,
    Path(path): Path<PluginArtifactPath>,
    Query(query): Query<PluginArtifactQuery>,
) -> Result<Response, StudioApiError> {
    let selector = PluginArtifactSelector::from(path);

    match query
        .format
        .unwrap_or(WendaoPluginArtifactOutputFormat::Json)
    {
        WendaoPluginArtifactOutputFormat::Json => render_plugin_artifact_json_response(&selector),
        WendaoPluginArtifactOutputFormat::Toml => render_plugin_artifact_toml_response(&selector),
    }
}

#[cfg(all(test, feature = "julia"))]
#[path = "../../../../../tests/unit/gateway/studio/router/handlers/capabilities/deployment.rs"]
mod tests;
