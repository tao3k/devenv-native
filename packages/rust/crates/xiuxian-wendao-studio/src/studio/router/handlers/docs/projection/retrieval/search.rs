use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::studio::router::handlers::docs::service::projection::retrieval::run_docs_retrieval;
use crate::studio::router::handlers::repo::parse::projection::parse_projection_page_kind;
use crate::studio::router::handlers::repo::parse::repo::required_registered_repo_id;
use crate::studio::router::handlers::repo::parse::search::required_search_query;
use crate::studio::router::handlers::repo::query::retrieval::RepoProjectedPageSearchApiQuery;
use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::DocsRetrievalQuery;

/// Docs retrieval endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `query` is missing, the kind filter is
/// invalid, repository lookup or analysis fails, or the background task
/// panics.
pub async fn retrieval(
    Query(query): Query<RepoProjectedPageSearchApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::DocsRetrievalResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let search_query = required_search_query(query.query.as_deref())?;
    let kind = parse_projection_page_kind(query.kind.as_deref())?;
    let limit = query.limit.unwrap_or(10).max(1);
    let result = run_docs_retrieval(
        Arc::clone(&state),
        DocsRetrievalQuery {
            repo_id,
            query: search_query,
            kind,
            limit,
        },
    )
    .await?;
    Ok(Json(result))
}
