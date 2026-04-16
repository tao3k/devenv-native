#[cfg(test)]
use crate::gateway::studio::router::StudioApiError;
#[cfg(test)]
use crate::gateway::studio::types::SearchHit;
#[cfg(test)]
use crate::search::SearchPlaneService;
#[cfg(test)]
use crate::search::repo_search::{
    search_repo_content_hits_for_query as shared_search_repo_content_hits_for_query,
    search_repo_entity_hits_for_query as shared_search_repo_entity_hits_for_query,
};

/// Search repo entity rows for a repo-scoped code query.
///
/// # Errors
///
/// Returns [`StudioApiError`] when the repo entity search plane fails.
#[cfg(test)]
pub(crate) async fn search_repo_entity_hits(
    search_plane: &SearchPlaneService,
    repo_id: &str,
    raw_query: &str,
    limit: usize,
) -> Result<Vec<SearchHit>, StudioApiError> {
    shared_search_repo_entity_hits_for_query(search_plane, repo_id, raw_query, limit)
        .await
        .map_err(|error| {
            StudioApiError::internal(
                "REPO_ENTITY_SEARCH_FAILED",
                "Failed to execute shared repo entity search",
                Some(error),
            )
        })
}

/// Search repo content rows for a repo-scoped code query.
///
/// # Errors
///
/// Returns [`StudioApiError`] when the repo content search plane fails.
#[cfg(test)]
pub(crate) async fn search_repo_content_hits(
    search_plane: &SearchPlaneService,
    repo_id: &str,
    raw_query: &str,
    limit: usize,
) -> Result<Vec<SearchHit>, StudioApiError> {
    shared_search_repo_content_hits_for_query(search_plane, repo_id, raw_query, limit)
        .await
        .map_err(|error| {
            StudioApiError::internal(
                "REPO_CONTENT_SEARCH_FAILED",
                "Failed to query repo content through the shared repo-search service",
                Some(error),
            )
        })
}

#[cfg(test)]
use crate::gateway::studio::router::StudioState;

/// Build repo entity search hits through the Studio state wrapper.
#[cfg(test)]
pub(crate) async fn build_repo_entity_search_hits(
    studio: &StudioState,
    repo_id: &str,
    raw_query: &str,
    limit: usize,
) -> Result<Vec<SearchHit>, StudioApiError> {
    search_repo_entity_hits(&studio.search_plane, repo_id, raw_query, limit).await
}

/// Build repo content search hits through the Studio state wrapper.
#[cfg(test)]
pub(crate) async fn build_repo_content_search_hits(
    studio: &StudioState,
    repo_id: &str,
    raw_query: &str,
    limit: usize,
) -> Result<Vec<SearchHit>, StudioApiError> {
    search_repo_content_hits(&studio.search_plane, repo_id, raw_query, limit).await
}
