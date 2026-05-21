//! Owns the Studio repo analysis overview surface.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::studio::router::handlers::repo::analysis::service::overview::run_repo_overview;
use crate::studio::router::handlers::repo::parse::source::required_registered_repo_id;
use crate::studio::router::handlers::repo::query::pages::RepoApiQuery;
use crate::studio::router::{GatewayState, StudioApiError};

/// Repository overview endpoint.
///
/// # Errors
///
/// Returns an error when `repo` is missing, repository lookup fails,
/// repository analysis fails, or the background task panics.
pub async fn overview(
    Query(query): Query<RepoApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::RepoOverviewResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let result = run_repo_overview(Arc::clone(&state), repo_id).await?;
    Ok(Json(result))
}
