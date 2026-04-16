use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::analyzers::{
    RepoProjectedPageIndexTreeSearchQuery, RepoProjectedPageSearchQuery,
    RepoProjectedRetrievalQuery,
};
use crate::gateway::studio::router::handlers::repo::projected_service::retrieval::{
    run_repo_projected_page_index_tree_search, run_repo_projected_page_search,
    run_repo_projected_retrieval,
};
use crate::gateway::studio::router::{GatewayState, StudioApiError};

use super::super::parse::projection::parse_projection_page_kind;
use super::super::parse::repo::required_registered_repo_id;
use super::super::parse::search::required_search_query;
use super::super::query::retrieval::RepoProjectedPageSearchApiQuery;

/// Projected page index tree search endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `query` is missing, the kind filter is
/// invalid, repository lookup or analysis fails, or the background task
/// panics.
pub async fn projected_page_index_tree_search(
    Query(query): Query<RepoProjectedPageSearchApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<crate::analyzers::RepoProjectedPageIndexTreeSearchResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let search_query = required_search_query(query.query.as_deref())?;
    let kind = parse_projection_page_kind(query.kind.as_deref())?;
    let limit = query.limit.unwrap_or(10).max(1);
    let result = run_repo_projected_page_index_tree_search(
        Arc::clone(&state),
        RepoProjectedPageIndexTreeSearchQuery {
            repo_id,
            query: search_query,
            kind,
            limit,
        },
    )
    .await?;
    Ok(Json(result))
}

/// Projected page search endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `query` is missing, the kind filter is
/// invalid, repository lookup or analysis fails, or the background task
/// panics.
pub async fn projected_page_search(
    Query(query): Query<RepoProjectedPageSearchApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<crate::analyzers::RepoProjectedPageSearchResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let search_query = required_search_query(query.query.as_deref())?;
    let kind = parse_projection_page_kind(query.kind.as_deref())?;
    let limit = query.limit.unwrap_or(10).max(1);
    let result = run_repo_projected_page_search(
        Arc::clone(&state),
        RepoProjectedPageSearchQuery {
            repo_id,
            query: search_query,
            kind,
            limit,
        },
    )
    .await?;
    Ok(Json(result))
}

/// Projected retrieval endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `query` is missing, the kind filter is
/// invalid, repository lookup or analysis fails, or the background task
/// panics.
pub async fn projected_retrieval(
    Query(query): Query<RepoProjectedPageSearchApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<crate::analyzers::RepoProjectedRetrievalResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let search_query = required_search_query(query.query.as_deref())?;
    let kind = parse_projection_page_kind(query.kind.as_deref())?;
    let limit = query.limit.unwrap_or(10).max(1);
    let result = run_repo_projected_retrieval(
        Arc::clone(&state),
        RepoProjectedRetrievalQuery {
            repo_id,
            query: search_query,
            kind,
            limit,
        },
    )
    .await?;
    Ok(Json(result))
}
