use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::studio::router::handlers::docs::service::projection::page::run_docs_page;
use crate::studio::router::handlers::repo::parse::repo::required_registered_repo_id;
use crate::studio::router::handlers::repo::parse::resource::required_page_id;
use crate::studio::router::handlers::repo::query::pages::RepoProjectedPageApiQuery;
use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::DocsPageQuery;

/// Docs page endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `page_id` is missing, repository lookup or
/// analysis fails, projected page lookup fails, or the background task panics.
pub async fn page(
    Query(query): Query<RepoProjectedPageApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::DocsPageResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let page_id = required_page_id(query.page_id.as_deref())?;
    let result = run_docs_page(Arc::clone(&state), DocsPageQuery { repo_id, page_id }).await?;
    Ok(Json(result))
}
