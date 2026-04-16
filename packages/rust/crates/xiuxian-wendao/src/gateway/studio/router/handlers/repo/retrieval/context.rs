use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::analyzers::{
    RepoProjectedRetrievalContextQuery, RepoProjectedRetrievalHitQuery,
};
use crate::gateway::studio::router::handlers::repo::projected_service::retrieval::{
    run_repo_projected_retrieval_context, run_repo_projected_retrieval_hit,
};
use crate::gateway::studio::router::{GatewayState, StudioApiError};

use super::super::parse::repo::required_registered_repo_id;
use super::super::parse::resource::required_page_id;
use super::super::query::retrieval::{
    RepoProjectedRetrievalContextApiQuery, RepoProjectedRetrievalHitApiQuery,
};

/// Projected retrieval hit endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `page_id` is missing, repository lookup or
/// analysis fails, hit lookup fails, or the background task panics.
pub async fn projected_retrieval_hit(
    Query(query): Query<RepoProjectedRetrievalHitApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<crate::analyzers::RepoProjectedRetrievalHitResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let page_id = required_page_id(query.page_id.as_deref())?;
    let node_id = query.node_id;
    let result = run_repo_projected_retrieval_hit(
        Arc::clone(&state),
        RepoProjectedRetrievalHitQuery {
            repo_id,
            page_id,
            node_id,
        },
    )
    .await?;
    Ok(Json(result))
}

/// Projected retrieval context endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `page_id` is missing, repository lookup or
/// analysis fails, retrieval context lookup fails, or the background task
/// panics.
pub async fn projected_retrieval_context(
    Query(query): Query<RepoProjectedRetrievalContextApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<crate::analyzers::RepoProjectedRetrievalContextResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let page_id = required_page_id(query.page_id.as_deref())?;
    let node_id = query.node_id;
    let related_limit = query.related_limit.unwrap_or(5);
    let result = run_repo_projected_retrieval_context(
        Arc::clone(&state),
        RepoProjectedRetrievalContextQuery {
            repo_id,
            page_id,
            node_id,
            related_limit,
        },
    )
    .await?;
    Ok(Json(result))
}
