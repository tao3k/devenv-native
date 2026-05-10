//! Owns the Studio repo analysis doc coverage surface.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::studio::router::handlers::repo::analysis::service::coverage::run_repo_doc_coverage;
use crate::studio::router::handlers::repo::parse::source::required_registered_repo_id;
use crate::studio::router::handlers::repo::query::analysis::RepoDocCoverageApiQuery;
use crate::studio::router::{GatewayState, StudioApiError};

/// Doc coverage endpoint.
///
/// # Errors
///
/// Returns an error when `repo` is missing, repository lookup or analysis
/// fails, or the background task panics.
pub async fn doc_coverage(
    Query(query): Query<RepoDocCoverageApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::DocCoverageResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let module_id = query.module_id;
    let result = run_repo_doc_coverage(Arc::clone(&state), repo_id, module_id).await?;
    Ok(Json(result))
}
