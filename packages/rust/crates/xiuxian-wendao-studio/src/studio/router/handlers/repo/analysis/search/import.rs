//! Owns the Studio analysis search import surface.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::studio::router::handlers::repo::analysis::search::service::imports::run_repo_import_search;
use crate::studio::router::handlers::repo::parse::search::required_import_search_filters;
use crate::studio::router::handlers::repo::parse::source::required_registered_repo_id;
use crate::studio::router::handlers::repo::query::analysis::RepoImportSearchApiQuery;
use crate::studio::router::{GatewayState, StudioApiError};

/// Import search endpoint.
///
/// # Errors
///
/// Returns an error when `repo` is missing, both `package` and `module` are
/// missing, repository lookup or analysis fails, or the background task panics.
pub async fn import_search(
    Query(query): Query<RepoImportSearchApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<xiuxian_wendao::analyzers::ImportSearchResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let (package, module) =
        required_import_search_filters(query.package.as_deref(), query.module.as_deref())?;
    let limit = query.limit.unwrap_or(10).max(1);
    let result =
        run_repo_import_search(Arc::clone(&state), repo_id, package, module, limit).await?;
    Ok(Json(result))
}

#[cfg(test)]
#[path = "../../../../../../../tests/unit/gateway/studio/router/handlers/repo/analysis/search/import.rs"]
mod tests;
