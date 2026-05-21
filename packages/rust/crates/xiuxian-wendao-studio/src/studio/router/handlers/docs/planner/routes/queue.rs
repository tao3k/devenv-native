//! Owns the Studio planner routes queue surface.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::studio::router::handlers::docs::service::planner::run_docs_planner_queue;
use crate::studio::router::handlers::docs::types::planner::DocsPlannerQueueApiQuery;
use crate::studio::router::handlers::repo::parse::projection::{
    parse_projected_gap_kind, parse_projection_page_kind,
};
use crate::studio::router::handlers::repo::parse::source::required_registered_repo_id;
use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::DocsPlannerQueueQuery;

/// Docs planner-queue endpoint.
///
/// # Errors
///
/// Returns an error when `repo` is missing, a filter is invalid, repository lookup or analysis
/// fails, or the background task panics.
pub async fn planner_queue(
    Query(query): Query<DocsPlannerQueueApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::DocsPlannerQueueResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let gap_kind = parse_projected_gap_kind(query.gap_kind.as_deref())?;
    let page_kind = parse_projection_page_kind(query.page_kind.as_deref())?;
    let per_kind_limit = query.per_kind_limit.unwrap_or(3).max(1);
    let result = run_docs_planner_queue(
        Arc::clone(&state),
        DocsPlannerQueueQuery {
            repo_id,
            gap_kind,
            page_kind,
            per_kind_limit,
        },
    )
    .await?;
    Ok(Json(result))
}
