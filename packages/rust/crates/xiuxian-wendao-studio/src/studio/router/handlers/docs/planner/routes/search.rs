use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::studio::router::handlers::docs::service::planner::run_docs_planner_search;
use crate::studio::router::handlers::docs::types::planner::DocsPlannerSearchApiQuery;
use crate::studio::router::handlers::repo::parse::projection::{
    parse_projected_gap_kind, parse_projection_page_kind,
};
use crate::studio::router::handlers::repo::parse::repo::required_registered_repo_id;
use crate::studio::router::handlers::repo::parse::search::required_search_query;
use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::DocsPlannerSearchQuery;

/// Docs planner-search endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `query` is missing, a filter is invalid, repository lookup or
/// analysis fails, or the background task panics.
pub async fn planner_search(
    Query(query): Query<DocsPlannerSearchApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::DocsPlannerSearchResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let search_query = required_search_query(query.query.as_deref())?;
    let gap_kind = parse_projected_gap_kind(query.gap_kind.as_deref())?;
    let page_kind = parse_projection_page_kind(query.page_kind.as_deref())?;
    let limit = query.limit.unwrap_or(10).max(1);
    let result = run_docs_planner_search(
        Arc::clone(&state),
        DocsPlannerSearchQuery {
            repo_id,
            query: search_query,
            gap_kind,
            page_kind,
            limit,
        },
    )
    .await?;
    Ok(Json(result))
}
