use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::gateway::studio::router::handlers::repo::command_service::run_repo_sync;
use crate::gateway::studio::router::handlers::repo::parse::repo::required_registered_repo_id;
use crate::gateway::studio::router::handlers::repo::parse::sync::parse_repo_sync_mode;
use crate::gateway::studio::router::handlers::repo::query::analysis::RepoSyncApiQuery;
use crate::gateway::studio::router::{GatewayState, StudioApiError};

/// Repo sync endpoint.
///
/// # Errors
///
/// Returns an error when `repo` is missing, the sync mode is invalid,
/// repository lookup fails, syncing fails, or the background task panics.
pub async fn sync(
    Query(query): Query<RepoSyncApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<crate::analyzers::RepoSyncResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let mode = parse_repo_sync_mode(query.mode.as_deref())?;
    let result = run_repo_sync(Arc::clone(&state), repo_id, mode).await?;
    Ok(Json(result))
}
