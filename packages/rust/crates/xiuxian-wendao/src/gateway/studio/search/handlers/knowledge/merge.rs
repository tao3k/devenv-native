use crate::gateway::studio::{
    StudioApiError, StudioState, configured_repositories, configured_repository,
    map_repo_intelligence_error,
};
use crate::gateway::studio::search::handlers::knowledge::IntentSearchTransportMetadata;
#[cfg(all(test, feature = "duckdb"))]
use crate::gateway::studio::search::handlers::knowledge::intent::configured_parquet_query_engine_label;
use crate::gateway::studio::types::SearchHit;
use crate::search::repo_search::search_repo_intent_outcome;

#[derive(Debug, Default)]
pub(super) struct RepoIntentMerge {
    pub(super) hits: Vec<SearchHit>,
    pub(super) transport: IntentSearchTransportMetadata,
    pub(super) pending_repos: Vec<String>,
    pub(super) skipped_repos: Vec<String>,
}

pub(super) async fn build_repo_intent_merge(
    studio: &StudioState,
    raw_query: &str,
    repo_hint: Option<&str>,
    limit: usize,
) -> Result<RepoIntentMerge, StudioApiError> {
    let repo_ids = if let Some(repo_id) = repo_hint {
        vec![
            configured_repository(studio, repo_id)
                .map_err(map_repo_intelligence_error)?
                .id,
        ]
    } else {
        configured_repositories(studio)
            .into_iter()
            .map(|repository| repository.id)
            .collect::<Vec<_>>()
    };
    #[cfg(all(test, feature = "duckdb"))]
    let has_repo_ids = !repo_ids.is_empty();

    let outcome = search_repo_intent_outcome(&studio.search_plane, repo_ids, raw_query, limit)
        .await
        .map_err(|error| {
            StudioApiError::internal(
                "REPO_INTENT_SEARCH_FAILED",
                "Failed to execute shared repo intent orchestration",
                Some(error),
            )
        })?;
    #[cfg(all(test, feature = "duckdb"))]
    let repo_query_engine = if has_repo_ids {
        Some(resolve_repo_intent_query_engine_label(studio)?)
    } else {
        None
    };
    Ok(RepoIntentMerge {
        transport: IntentSearchTransportMetadata {
            #[cfg(test)]
            knowledge_query_engine: None,
            #[cfg(test)]
            local_symbol_query_engine: None,
            #[cfg(all(test, feature = "duckdb"))]
            repo_query_engine,
            #[cfg(test)]
            repo_content_transport: outcome.repo_content_available.then_some("flight_contract"),
        },
        hits: outcome.hits,
        pending_repos: outcome.pending_repos,
        skipped_repos: outcome.skipped_repos,
    })
}

#[cfg(all(test, feature = "duckdb"))]
fn resolve_repo_intent_query_engine_label(
    studio: &StudioState,
) -> Result<&'static str, StudioApiError> {
    configured_parquet_query_engine_label(&studio.search_plane).map_err(|error| {
        StudioApiError::internal(
            "REPO_INTENT_QUERY_ENGINE_FAILED",
            "Failed to resolve repo-intent query-engine metadata",
            Some(error),
        )
    })
}
