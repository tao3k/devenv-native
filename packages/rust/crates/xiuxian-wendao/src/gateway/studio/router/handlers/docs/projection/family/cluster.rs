use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::analyzers::DocsFamilyClusterQuery;
use crate::gateway::studio::router::handlers::docs::service::projection::family::run_docs_family_cluster;
use crate::gateway::studio::router::handlers::repo::parse::projection::required_projection_page_kind;
use crate::gateway::studio::router::handlers::repo::parse::repo::required_registered_repo_id;
use crate::gateway::studio::router::handlers::repo::parse::resource::required_page_id;
use crate::gateway::studio::router::handlers::repo::query::family::RepoProjectedPageFamilyClusterApiQuery;
use crate::gateway::studio::router::{GatewayState, StudioApiError};

/// Docs family cluster endpoint.
///
/// # Errors
///
/// Returns an error when `repo`, `page_id`, or `kind` is missing or invalid,
/// repository lookup or analysis fails, family-cluster lookup fails, or the
/// background task panics.
pub async fn family_cluster(
    Query(query): Query<RepoProjectedPageFamilyClusterApiQuery>,
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<crate::analyzers::DocsFamilyClusterResult>, StudioApiError> {
    let repo_id = required_registered_repo_id(state.studio.as_ref(), query.repo.as_deref())?;
    let page_id = required_page_id(query.page_id.as_deref())?;
    let kind = required_projection_page_kind(query.kind.as_deref())?;
    let limit = query.limit.unwrap_or(3).max(1);
    let result = run_docs_family_cluster(
        Arc::clone(&state),
        DocsFamilyClusterQuery {
            repo_id,
            page_id,
            kind,
            limit,
        },
    )
    .await?;
    Ok(Json(result))
}
