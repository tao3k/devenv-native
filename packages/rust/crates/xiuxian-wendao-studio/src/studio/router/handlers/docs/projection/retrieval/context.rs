use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::studio::router::handlers::docs::service::projection::retrieval::{
    run_docs_retrieval_context, run_docs_retrieval_hit,
};
use crate::studio::router::handlers::repo::parse::repo::required_registered_repo_id;
use crate::studio::router::handlers::repo::parse::resource::required_page_id;
use crate::studio::router::handlers::repo::query::retrieval::{
    RepoProjectedRetrievalContextApiQuery, RepoProjectedRetrievalHitApiQuery,
};
use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::{DocsRetrievalContextQuery, DocsRetrievalHitQuery};

/// Docs retrieval context endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `page_id` is missing, repository lookup or
/// analysis fails, retrieval context lookup fails, or the background task
/// panics.
pub async fn retrieval_context(
    Query(query): Query<RepoProjectedRetrievalContextApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::DocsRetrievalContextResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let page_id = required_page_id(query.page_id.as_deref())?;
    let node_id = query.node_id;
    let related_limit = query.related_limit.unwrap_or(5);
    let result = run_docs_retrieval_context(
        Arc::clone(&state),
        DocsRetrievalContextQuery {
            repo_id,
            page_id,
            node_id,
            related_limit,
        },
    )
    .await?;
    Ok(Json(result))
}

/// Docs retrieval hit endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `page_id` is missing, repository lookup or
/// analysis fails, retrieval-hit lookup fails, or the background task panics.
pub async fn retrieval_hit(
    Query(query): Query<RepoProjectedRetrievalHitApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::DocsRetrievalHitResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let page_id = required_page_id(query.page_id.as_deref())?;
    let node_id = query.node_id;
    let result = run_docs_retrieval_hit(
        Arc::clone(&state),
        DocsRetrievalHitQuery {
            repo: repo_id,
            page: page_id,
            node: node_id,
        },
    )
    .await?;
    Ok(Json(result))
}
