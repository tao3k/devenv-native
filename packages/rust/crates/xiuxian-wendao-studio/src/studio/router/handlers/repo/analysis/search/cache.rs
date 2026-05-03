use std::future::Future;

use crate::studio::router::StudioApiError;
use xiuxian_wendao::analyzers::{RepositoryAnalysisCacheKey, RepositorySearchQueryCacheKey};
use xiuxian_wendao::search::FuzzySearchOptions;
use xiuxian_wendao::search::{RepoSearchQueryCacheKeyInput, SearchCorpusKind, SearchPlaneCacheTtl};

pub(crate) async fn with_cached_repo_search_result<T, F, Fut>(
    search_plane: &xiuxian_wendao::search::SearchPlaneService,
    scope: &'static str,
    repo_id: &str,
    query: &str,
    limit: usize,
    load: F,
) -> Result<T, StudioApiError>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, StudioApiError>>,
{
    let cache_key = repo_search_cache_key(search_plane, scope, repo_id, query, limit).await;
    if let Some(cache_key) = cache_key.as_ref()
        && let Some(cached) = search_plane.cache_get_json::<T>(cache_key).await
    {
        return Ok(cached);
    }
    let result = load().await?;
    if let Some(cache_key) = cache_key.as_ref() {
        search_plane
            .cache_set_json(cache_key, SearchPlaneCacheTtl::HotQuery, &result)
            .await;
    }
    Ok(result)
}

async fn repo_search_cache_key(
    search_plane: &xiuxian_wendao::search::SearchPlaneService,
    scope: &'static str,
    repo_id: &str,
    query: &str,
    limit: usize,
) -> Option<String> {
    let repo_ids = [repo_id.to_string()];
    search_plane
        .repo_search_query_cache_key(RepoSearchQueryCacheKeyInput {
            scope,
            corpora: &[],
            repo_corpora: &[SearchCorpusKind::RepoEntity],
            repo_ids: &repo_ids,
            query,
            limit,
            intent: None,
            repo_hint: Some(repo_id),
        })
        .await
}

pub(crate) fn repository_search_key(
    cached_cache_key: &RepositoryAnalysisCacheKey,
    scope: &'static str,
    query: &str,
    limit: usize,
    options: FuzzySearchOptions,
) -> RepositorySearchQueryCacheKey {
    RepositorySearchQueryCacheKey::new(cached_cache_key, scope, query, None, options, limit)
}
