use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::studio::router::handlers::docs::service::planner::run_docs_planner_item;
use crate::studio::router::handlers::docs::types::planner::DocsPlannerItemApiQuery;
use crate::studio::router::handlers::repo::parse::projection::parse_projection_page_kind;
use crate::studio::router::handlers::repo::parse::repo::required_registered_repo_id;
use crate::studio::router::handlers::repo::parse::resource::required_gap_id;
use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::DocsPlannerItemQuery;

/// Docs planner-item endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `gap_id` is missing, the family filter is invalid,
/// repository lookup or analysis fails, planner-item lookup fails, or the background task panics.
pub async fn planner_item(
    Query(query): Query<DocsPlannerItemApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::DocsPlannerItemResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let gap_id = required_gap_id(query.gap_id.as_deref())?;
    let family_kind = parse_projection_page_kind(query.family_kind.as_deref())?;
    let related_limit = query.related_limit.unwrap_or(5);
    let family_limit = query.family_limit.unwrap_or(3).max(1);
    let result = run_docs_planner_item(
        Arc::clone(&state),
        DocsPlannerItemQuery {
            repo_id,
            gap_id,
            family_kind,
            related_limit,
            family_limit,
        },
    )
    .await?;
    Ok(Json(result))
}
