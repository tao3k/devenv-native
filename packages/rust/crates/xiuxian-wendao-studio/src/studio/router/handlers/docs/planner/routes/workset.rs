use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::studio::router::handlers::docs::service::planner::run_docs_planner_workset;
use crate::studio::router::handlers::docs::types::planner::DocsPlannerWorksetApiQuery;
use crate::studio::router::handlers::repo::parse::projection::{
    parse_projected_gap_kind, parse_projection_page_kind,
};
use crate::studio::router::handlers::repo::parse::repo::required_registered_repo_id;
use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::DocsPlannerWorksetQuery;

/// Docs planner-workset endpoint.
///
/// # Errors
///
/// Returns an error when `repo` is missing, a filter is invalid, repository lookup or analysis
/// fails, one selected planner item cannot be reopened, or the background task panics.
pub async fn planner_workset(
    Query(query): Query<DocsPlannerWorksetApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::DocsPlannerWorksetResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let gap_kind = parse_projected_gap_kind(query.gap_kind.as_deref())?;
    let page_kind = parse_projection_page_kind(query.page_kind.as_deref())?;
    let per_kind_limit = query.per_kind_limit.unwrap_or(3).max(1);
    let limit = query.limit.unwrap_or(3).max(1);
    let family_kind = parse_projection_page_kind(query.family_kind.as_deref())?;
    let related_limit = query.related_limit.unwrap_or(5);
    let family_limit = query.family_limit.unwrap_or(3).max(1);
    let result = run_docs_planner_workset(
        Arc::clone(&state),
        DocsPlannerWorksetQuery {
            repo_id,
            gap_kind,
            page_kind,
            per_kind_limit,
            limit,
            family_kind,
            related_limit,
            family_limit,
        },
    )
    .await?;
    Ok(Json(result))
}
