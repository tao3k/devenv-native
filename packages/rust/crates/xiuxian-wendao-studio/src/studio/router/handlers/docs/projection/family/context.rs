//! Owns the Studio projection family context surface.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::studio::router::handlers::docs::service::projection::family::{
    run_docs_family_context, run_docs_family_search,
};
use crate::studio::router::handlers::repo::parse::projection::parse_projection_page_kind;
use crate::studio::router::handlers::repo::parse::resource::required_page_id;
use crate::studio::router::handlers::repo::parse::search::required_search_query;
use crate::studio::router::handlers::repo::parse::source::required_registered_repo_id;
use crate::studio::router::handlers::repo::query::family::{
    RepoProjectedPageFamilyContextApiQuery, RepoProjectedPageFamilySearchApiQuery,
};
use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::{DocsFamilyContextQuery, DocsFamilySearchQuery};

/// Docs family context endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `page_id` is missing, repository lookup or
/// analysis fails, family-context lookup fails, or the background task panics.
pub async fn family_context(
    Query(query): Query<RepoProjectedPageFamilyContextApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::DocsFamilyContextResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let page_id = required_page_id(query.page_id.as_deref())?;
    let per_kind_limit = query.per_kind_limit.unwrap_or(3);
    let result = run_docs_family_context(
        Arc::clone(&state),
        DocsFamilyContextQuery {
            repo_id,
            page_id,
            per_kind_limit,
        },
    )
    .await?;
    Ok(Json(result))
}

/// Docs family search endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `query` is missing, the kind filter is
/// invalid, repository lookup or analysis fails, or the background task panics.
pub async fn family_search(
    Query(query): Query<RepoProjectedPageFamilySearchApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::DocsFamilySearchResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let search_query = required_search_query(query.query.as_deref())?;
    let kind = parse_projection_page_kind(query.kind.as_deref())?;
    let limit = query.limit.unwrap_or(10).max(1);
    let per_kind_limit = query.per_kind_limit.unwrap_or(3);
    let result = run_docs_family_search(
        Arc::clone(&state),
        DocsFamilySearchQuery {
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
