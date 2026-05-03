use std::sync::Arc;

use crate::studio::router::GatewayState;
use crate::studio::router::StudioApiError;
use crate::studio::router::handlers::repo::shared::execution::with_repo_cached_analysis_bundle;
use xiuxian_wendao::analyzers::{
    RepoIntelligenceError, RepoProjectedPageIndexTreeSearchQuery,
    RepoProjectedPageIndexTreeSearchResult, RepoProjectedPageSearchQuery,
    RepoProjectedPageSearchResult, RepoProjectedRetrievalContextQuery,
    RepoProjectedRetrievalContextResult, RepoProjectedRetrievalHitQuery,
    RepoProjectedRetrievalHitResult, RepoProjectedRetrievalQuery, RepoProjectedRetrievalResult,
    RepositorySearchQueryCacheKey, build_repo_projected_page_index_tree_search,
    build_repo_projected_page_search_with_artifacts, build_repo_projected_retrieval,
    build_repo_projected_retrieval_context, build_repo_projected_retrieval_hit,
    load_cached_repository_search_result, repository_search_artifacts,
    store_cached_repository_search_result,
};
use xiuxian_wendao::search::FuzzySearchOptions;

use super::analysis::run_repo_projected_analysis;

pub(crate) async fn run_repo_projected_retrieval_hit(
    state: Arc<GatewayState>,
    query: RepoProjectedRetrievalHitQuery,
) -> Result<RepoProjectedRetrievalHitResult, StudioApiError> {
    run_repo_projected_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "REPO_PROJECTED_RETRIEVAL_HIT_PANIC",
        "Repo projected retrieval hit task failed unexpectedly",
        move |analysis| build_repo_projected_retrieval_hit(&query, &analysis),
    )
    .await
}

pub(crate) async fn run_repo_projected_retrieval_context(
    state: Arc<GatewayState>,
    query: RepoProjectedRetrievalContextQuery,
) -> Result<RepoProjectedRetrievalContextResult, StudioApiError> {
    run_repo_projected_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "REPO_PROJECTED_RETRIEVAL_CONTEXT_PANIC",
        "Repo projected retrieval context task failed unexpectedly",
        move |analysis| build_repo_projected_retrieval_context(&query, &analysis),
    )
    .await
}

pub(crate) async fn run_repo_projected_page_index_tree_search(
    state: Arc<GatewayState>,
    query: RepoProjectedPageIndexTreeSearchQuery,
) -> Result<RepoProjectedPageIndexTreeSearchResult, StudioApiError> {
    run_repo_projected_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "REPO_PROJECTED_PAGE_INDEX_TREE_SEARCH_PANIC",
        "Repo projected page-index tree search task failed unexpectedly",
        move |analysis| {
            Ok::<_, RepoIntelligenceError>(build_repo_projected_page_index_tree_search(
                &query, &analysis,
            ))
        },
    )
    .await
}

pub(crate) async fn run_repo_projected_page_search(
    state: Arc<GatewayState>,
    query: RepoProjectedPageSearchQuery,
) -> Result<RepoProjectedPageSearchResult, StudioApiError> {
    with_repo_cached_analysis_bundle(
        Arc::clone(&state),
        query.repo_id.clone(),
        "REPO_PROJECTED_PAGE_SEARCH_PANIC",
        "Repo projected page search task failed unexpectedly",
        move |cached| {
            let filter = query
                .kind
                .map(|kind| format!("{kind:?}").to_ascii_lowercase());
            let cache_key = RepositorySearchQueryCacheKey::new(
                &cached.cache_key,
                "repo.projected-page-search",
                query.query.as_str(),
                filter,
                FuzzySearchOptions::document_search(),
                query.limit,
            );
            if let Some(result) = load_cached_repository_search_result(&cache_key)? {
                return Ok(result);
            }

            let artifacts = repository_search_artifacts(&cached.cache_key, &cached.analysis)?;
            let result = build_repo_projected_page_search_with_artifacts(
                &query,
                &cached.analysis,
                artifacts.as_ref(),
            );
            store_cached_repository_search_result(&cache_key, &result)?;
            Ok::<_, RepoIntelligenceError>(result)
        },
    )
    .await
}

pub(crate) async fn run_repo_projected_retrieval(
    state: Arc<GatewayState>,
    query: RepoProjectedRetrievalQuery,
) -> Result<RepoProjectedRetrievalResult, StudioApiError> {
    run_repo_projected_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "REPO_PROJECTED_RETRIEVAL_PANIC",
        "Repo projected retrieval task failed unexpectedly",
        move |analysis| {
            Ok::<_, RepoIntelligenceError>(build_repo_projected_retrieval(&query, &analysis))
        },
    )
    .await
}
