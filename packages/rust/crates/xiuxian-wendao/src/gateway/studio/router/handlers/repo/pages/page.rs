use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::analyzers::RepoProjectedPageQuery;
use crate::gateway::studio::router::handlers::repo::projected_service::pages::run_repo_projected_page;
use crate::gateway::studio::router::{GatewayState, StudioApiError};

use crate::gateway::studio::router::handlers::repo::parse::repo::required_registered_repo_id;
use crate::gateway::studio::router::handlers::repo::parse::resource::required_page_id;
use crate::gateway::studio::router::handlers::repo::query::pages::RepoProjectedPageApiQuery;

/// Projected page endpoint.
///
/// # Errors
///
/// Returns an error when `repo` or `page_id` is missing, repository lookup or
/// analysis fails, projected page lookup fails, or the background task panics.
pub async fn projected_page(
    Query(query): Query<RepoProjectedPageApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<crate::analyzers::RepoProjectedPageResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let page_id = required_page_id(query.page_id.as_deref())?;
    let result = run_repo_projected_page(
        Arc::clone(&state),
        RepoProjectedPageQuery { repo_id, page_id },
    )
    .await?;
    Ok(Json(result))
}
