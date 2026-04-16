use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::analyzers::{RepoProjectedPageNavigationQuery, RepoProjectedPageNavigationSearchQuery};
use crate::gateway::studio::router::handlers::repo::projected_service::family::{
    run_repo_projected_page_navigation, run_repo_projected_page_navigation_search,
};
use crate::gateway::studio::router::{GatewayState, StudioApiError};

use super::super::parse::projection::parse_projection_page_kind;
use super::super::parse::repo::required_registered_repo_id;
use super::super::parse::resource::required_page_id;
use super::super::parse::search::required_search_query;
use super::super::query::family::{
    RepoProjectedPageNavigationApiQuery, RepoProjectedPageNavigationSearchApiQuery,
};

/// Projected page navigation endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `page_id` is missing, the family kind is
/// invalid, repository lookup or analysis fails, navigation bundle lookup
/// fails, or the background task panics.
pub async fn projected_page_navigation(
    Query(query): Query<RepoProjectedPageNavigationApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<crate::analyzers::RepoProjectedPageNavigationResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let page_id = required_page_id(query.page_id.as_deref())?;
    let node_id = query.node_id;
    let family_kind = parse_projection_page_kind(query.family_kind.as_deref())?;
    let related_limit = query.related_limit.unwrap_or(5);
    let family_limit = query.family_limit.unwrap_or(3).max(1);
    let result = run_repo_projected_page_navigation(
        Arc::clone(&state),
        RepoProjectedPageNavigationQuery {
            repo_id,
            page_id,
            node_id,
            family_kind,
            related_limit,
            family_limit,
        },
    )
    .await?;
    Ok(Json(result))
}

/// Projected page navigation search endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `query` is missing, a page-kind filter is
/// invalid, repository lookup or analysis fails, or the background task
/// panics.
pub async fn projected_page_navigation_search(
    Query(query): Query<RepoProjectedPageNavigationSearchApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<crate::analyzers::RepoProjectedPageNavigationSearchResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let search_query = required_search_query(query.query.as_deref())?;
    let kind = parse_projection_page_kind(query.kind.as_deref())?;
    let family_kind = parse_projection_page_kind(query.family_kind.as_deref())?;
    let limit = query.limit.unwrap_or(10).max(1);
    let related_limit = query.related_limit.unwrap_or(5);
    let family_limit = query.family_limit.unwrap_or(3).max(1);
    let result = run_repo_projected_page_navigation_search(
        Arc::clone(&state),
        RepoProjectedPageNavigationSearchQuery {
            repo_id,
            query: search_query,
            kind,
            family_kind,
            limit,
            related_limit,
            family_limit,
        },
    )
    .await?;
    Ok(Json(result))
}
