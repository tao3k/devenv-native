//! Owns the Studio handlers repo index surface.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};
use serde::Serialize;

use crate::studio::router::handlers::repo::analysis::index_status_flight::repo_index_status_response_with_diagnostics;
use crate::studio::router::handlers::repo::command_service::{
    run_repo_index, run_repo_index_status,
};
use crate::studio::router::{
    GatewayState, StudioApiError, StudioBootstrapBackgroundIndexingTelemetry,
};
use xiuxian_wendao::repo_index::{RepoIndexRequest, RepoIndexStatusResponse};

use super::query::analysis::RepoIndexStatusApiQuery;

/// Repo-index status payload enriched with bootstrap-indexing telemetry.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoIndexStatusEnvelope {
    #[serde(flatten)]
    status: RepoIndexStatusResponse,
    #[serde(flatten)]
    telemetry: StudioBootstrapBackgroundIndexingTelemetry,
}

/// Repo index enqueue endpoint.
///
/// # Errors
///
/// Returns an error when a requested repository cannot be resolved or when no
/// configured repository is available for indexing.
pub async fn repo_index(
    State(state): State<Arc<GatewayState>>,
    Json(payload): Json<RepoIndexRequest>,
) -> Result<Json<RepoIndexStatusResponse>, StudioApiError> {
    Ok(Json(run_repo_index(Arc::clone(&state), payload).await?))
}

/// Repo index status endpoint.
///
/// # Errors
///
/// This handler currently does not produce handler-local errors.
pub async fn repo_index_status(
    Query(query): Query<RepoIndexStatusApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<RepoIndexStatusEnvelope>, StudioApiError> {
    let status = run_repo_index_status(&state, query.repo.as_deref());
    let status = repo_index_status_payload_with_diagnostics(status).await;
    let telemetry = state.studio.bootstrap_background_indexing_telemetry();
    Ok(Json(RepoIndexStatusEnvelope { status, telemetry }))
}

pub(crate) async fn repo_index_status_payload_with_diagnostics(
    status: RepoIndexStatusResponse,
) -> RepoIndexStatusResponse {
    repo_index_status_response_with_diagnostics(&status).await
}

#[cfg(test)]
#[path = "../../../../../tests/unit/gateway/studio/router/handlers/repo/index.rs"]
mod tests;
