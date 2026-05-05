use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::studio::router::handlers::docs::service::planner::run_docs_planner_rank;
use crate::studio::router::handlers::docs::types::planner::DocsPlannerRankApiQuery;
use crate::studio::router::handlers::repo::parse::projection::{
    parse_projected_gap_kind, parse_projection_page_kind,
};
use crate::studio::router::handlers::repo::parse::repo::required_registered_repo_id;
use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::DocsPlannerRankQuery;

/// Docs planner-rank endpoint.
///
/// # Errors
///
/// Returns an error when `repo` is missing, a filter is invalid, repository lookup or analysis
/// fails, or the background task panics.
pub async fn planner_rank(
    Query(query): Query<DocsPlannerRankApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::DocsPlannerRankResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let gap_kind = parse_projected_gap_kind(query.gap_kind.as_deref())?;
    let page_kind = parse_projection_page_kind(query.page_kind.as_deref())?;
    let limit = query.limit.unwrap_or(10).max(1);
    let result = run_docs_planner_rank(
        Arc::clone(&state),
        DocsPlannerRankQuery {
            repo_id,
            gap_kind,
            page_kind,
            limit,
        },
    )
    .await?;
    Ok(Json(result))
}
