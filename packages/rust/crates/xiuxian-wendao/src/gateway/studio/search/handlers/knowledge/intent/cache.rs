use crate::gateway::studio::{
    StudioApiError, StudioState, configured_repositories, configured_repository,
    map_repo_intelligence_error,
};
use crate::search::{RepoSearchQueryCacheKeyInput, SearchCorpusKind};

pub(crate) async fn build_intent_cache_key(
    studio: &StudioState,
    raw_query: &str,
    query_text: &str,
    repo_hint: Option<&str>,
    limit: usize,
    intent: Option<&str>,
    code_biased: bool,
) -> Result<Option<String>, StudioApiError> {
    if code_biased {
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
        return Ok(studio
            .search_plane
            .repo_search_query_cache_key(RepoSearchQueryCacheKeyInput {
                scope: "intent_hybrid_code",
                corpora: &[
                    SearchCorpusKind::KnowledgeSection,
                    SearchCorpusKind::LocalSymbol,
                ],
                repo_corpora: &[
                    SearchCorpusKind::RepoEntity,
                    SearchCorpusKind::RepoContentChunk,
                ],
                repo_ids: repo_ids.as_slice(),
                query: raw_query,
                limit,
                intent,
                repo_hint,
            })
            .await);
    }

    Ok(studio.search_plane.search_query_cache_key(
        "intent_hybrid",
        &[
            SearchCorpusKind::KnowledgeSection,
            SearchCorpusKind::LocalSymbol,
        ],
        query_text,
        limit,
        intent,
        None,
    ))
}
