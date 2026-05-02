use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::analyzers::{
    RepoProjectedGapReportQuery, RepoProjectedPageIndexTreesQuery, RepoProjectedPagesQuery,
};
use crate::gateway::studio::router::handlers::repo::projected_service::collection::{
    run_repo_projected_gap_report, run_repo_projected_page_index_trees, run_repo_projected_pages,
};
use crate::gateway::studio::router::{GatewayState, StudioApiError};

use crate::gateway::studio::router::handlers::repo::parse::repo::required_registered_repo_id;
use crate::gateway::studio::router::handlers::repo::query::pages::RepoApiQuery;

/// Projected pages endpoint.
///
/// # Errors
///
/// Returns an error when `repo` is missing, repository lookup or analysis
/// fails, or the background task panics.
pub async fn projected_pages(
    Query(query): Query<RepoApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<crate::analyzers::RepoProjectedPagesResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let result =
        run_repo_projected_pages(Arc::clone(&state), RepoProjectedPagesQuery { repo_id }).await?;
    Ok(Json(result))
}

/// Projected gap report endpoint.
///
/// # Errors
///
/// Returns an error when `repo` is missing, repository lookup or analysis
/// fails, or the background task panics.
pub async fn projected_gap_report(
    Query(query): Query<RepoApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<crate::analyzers::RepoProjectedGapReportResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let result =
        run_repo_projected_gap_report(Arc::clone(&state), RepoProjectedGapReportQuery { repo_id })
            .await?;
    Ok(Json(result))
}

/// Projected page index trees endpoint.
///
/// # Errors
///
/// Returns an error when `repo` is missing, repository lookup or analysis
/// fails, page-index tree construction fails, or the background task panics.
pub async fn projected_page_index_trees(
    Query(query): Query<RepoApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<crate::analyzers::RepoProjectedPageIndexTreesResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let result = run_repo_projected_page_index_trees(
        Arc::clone(&state),
        RepoProjectedPageIndexTreesQuery { repo_id },
    )
    .await?;
    Ok(Json(result))
}
