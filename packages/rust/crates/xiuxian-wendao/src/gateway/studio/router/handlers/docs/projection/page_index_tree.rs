use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::analyzers::DocsPageIndexTreeQuery;
use crate::gateway::studio::router::handlers::docs::service::projection::page::run_docs_page_index_tree;
use crate::gateway::studio::router::handlers::repo::parse::repo::required_registered_repo_id;
use crate::gateway::studio::router::handlers::repo::parse::resource::required_page_id;
use crate::gateway::studio::router::handlers::repo::query::pages::RepoProjectedPageApiQuery;
use crate::gateway::studio::router::{GatewayState, StudioApiError};

/// Docs page-index tree endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `page_id` is missing, repository lookup or
/// analysis fails, page-index tree lookup fails, or the background task
/// panics.
pub async fn page_index_tree(
    Query(query): Query<RepoProjectedPageApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<crate::analyzers::DocsPageIndexTreeResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let page_id = required_page_id(query.page_id.as_deref())?;
    let result = run_docs_page_index_tree(
        Arc::clone(&state),
        DocsPageIndexTreeQuery { repo_id, page_id },
    )
    .await?;
    Ok(Json(result))
}
