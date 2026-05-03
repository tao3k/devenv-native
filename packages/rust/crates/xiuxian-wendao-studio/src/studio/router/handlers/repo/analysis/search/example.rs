use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::studio::router::handlers::repo::analysis::search::service::typed::run_repo_example_search;
use crate::studio::router::handlers::repo::parse::repo::required_registered_repo_id;
use crate::studio::router::handlers::repo::parse::search::required_search_query;
use crate::studio::router::handlers::repo::query::analysis::RepoSearchApiQuery;
use crate::studio::router::{GatewayState, StudioApiError};

/// Example search endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `query` is missing, repository lookup or
/// analysis fails, or the background task panics.
pub async fn example_search(
    Query(query): Query<RepoSearchApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::ExampleSearchResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let search_query = required_search_query(query.query.as_deref())?;
    let limit = query.limit.unwrap_or(10).max(1);
    let result = run_repo_example_search(Arc::clone(&state), repo_id, search_query, limit).await?;
    Ok(Json(result))
}
