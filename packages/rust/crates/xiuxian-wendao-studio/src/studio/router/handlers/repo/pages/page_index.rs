//! Owns the Studio repo pages page index surface.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::studio::router::handlers::repo::projected_service::pages::{
    run_repo_projected_page_index_node, run_repo_projected_page_index_tree,
};
use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::{RepoProjectedPageIndexNodeQuery, RepoProjectedPageIndexTreeQuery};

use crate::studio::router::handlers::repo::parse::resource::{required_node_id, required_page_id};
use crate::studio::router::handlers::repo::parse::source::required_registered_repo_id;
use crate::studio::router::handlers::repo::query::pages::{
    RepoProjectedPageApiQuery, RepoProjectedPageIndexNodeApiQuery,
};

/// Projected page index tree endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `page_id` is missing, repository lookup or
/// analysis fails, page-index tree lookup fails, or the background task
/// panics.
pub async fn projected_page_index_tree(
    Query(query): Query<RepoProjectedPageApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::RepoProjectedPageIndexTreeResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let page_id = required_page_id(query.page_id.as_deref())?;
    let result = run_repo_projected_page_index_tree(
        Arc::clone(&state),
        RepoProjectedPageIndexTreeQuery { repo_id, page_id },
    )
    .await?;
    Ok(Json(result))
}

/// Projected page index node endpoint.
///
/// # Errors
///
/// Returns an error when `repo`, `page_id`, or `node_id` is missing,
/// repository lookup or analysis fails, page-index node lookup fails, or the
/// background task panics.
pub async fn projected_page_index_node(
    Query(query): Query<RepoProjectedPageIndexNodeApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::RepoProjectedPageIndexNodeResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let page_id = required_page_id(query.page_id.as_deref())?;
    let node_id = required_node_id(query.node_id.as_deref())?;
    let result = run_repo_projected_page_index_node(
        Arc::clone(&state),
        RepoProjectedPageIndexNodeQuery {
            repo_id,
            page_id,
            node_id,
        },
    )
    .await?;
    Ok(Json(result))
}
