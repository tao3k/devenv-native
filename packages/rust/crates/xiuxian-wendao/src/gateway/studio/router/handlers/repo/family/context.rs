use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::analyzers::{
    RepoProjectedPageFamilyClusterQuery, RepoProjectedPageFamilyContextQuery,
    RepoProjectedPageFamilySearchQuery,
};
use crate::gateway::studio::router::handlers::repo::projected_service::family::{
    run_repo_projected_page_family_cluster, run_repo_projected_page_family_context,
    run_repo_projected_page_family_search,
};
use crate::gateway::studio::router::{GatewayState, StudioApiError};

use super::super::parse::projection::{parse_projection_page_kind, required_projection_page_kind};
use super::super::parse::repo::required_registered_repo_id;
use super::super::parse::resource::required_page_id;
use super::super::parse::search::required_search_query;
use super::super::query::family::{
    RepoProjectedPageFamilyClusterApiQuery, RepoProjectedPageFamilyContextApiQuery,
    RepoProjectedPageFamilySearchApiQuery,
};

/// Projected page family context endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `page_id` is missing, repository lookup or
/// analysis fails, family context lookup fails, or the background task panics.
pub async fn projected_page_family_context(
    Query(query): Query<RepoProjectedPageFamilyContextApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<crate::analyzers::RepoProjectedPageFamilyContextResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let page_id = required_page_id(query.page_id.as_deref())?;
    let per_kind_limit = query.per_kind_limit.unwrap_or(3);
    let result = run_repo_projected_page_family_context(
        Arc::clone(&state),
        RepoProjectedPageFamilyContextQuery {
            repo_id,
            page_id,
            per_kind_limit,
        },
    )
    .await?;
    Ok(Json(result))
}

/// Projected page family search endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `query` is missing, the kind filter is
/// invalid, repository lookup or analysis fails, or the background task
/// panics.
pub async fn projected_page_family_search(
    Query(query): Query<RepoProjectedPageFamilySearchApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<crate::analyzers::RepoProjectedPageFamilySearchResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let search_query = required_search_query(query.query.as_deref())?;
    let kind = parse_projection_page_kind(query.kind.as_deref())?;
    let limit = query.limit.unwrap_or(10).max(1);
    let per_kind_limit = query.per_kind_limit.unwrap_or(3);
    let result = run_repo_projected_page_family_search(
        Arc::clone(&state),
        RepoProjectedPageFamilySearchQuery {
            repo_id,
            query: search_query,
            kind,
            limit,
            per_kind_limit,
        },
    )
    .await?;
    Ok(Json(result))
}

/// Projected page family cluster endpoint.
///
/// # Errors
///
/// Returns an error when `repo`, `page_id`, or `kind` is missing or invalid,
/// repository lookup or analysis fails, family cluster lookup fails, or the
/// background task panics.
pub async fn projected_page_family_cluster(
    Query(query): Query<RepoProjectedPageFamilyClusterApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<crate::analyzers::RepoProjectedPageFamilyClusterResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let page_id = required_page_id(query.page_id.as_deref())?;
    let kind = required_projection_page_kind(query.kind.as_deref())?;
    let limit = query.limit.unwrap_or(3).max(1);
    let result = run_repo_projected_page_family_cluster(
        Arc::clone(&state),
        RepoProjectedPageFamilyClusterQuery {
            repo_id,
            page_id,
            kind,
            limit,
        },
    )
    .await?;
    Ok(Json(result))
}
