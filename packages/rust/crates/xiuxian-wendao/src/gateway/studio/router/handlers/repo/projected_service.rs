use std::sync::Arc;

use crate::analyzers::{
    RepoIntelligenceError, RepoProjectedGapReportQuery, RepoProjectedGapReportResult,
    RepoProjectedPageFamilyClusterQuery, RepoProjectedPageFamilyClusterResult,
    RepoProjectedPageFamilyContextQuery, RepoProjectedPageFamilyContextResult,
    RepoProjectedPageFamilySearchQuery, RepoProjectedPageFamilySearchResult,
    RepoProjectedPageIndexNodeQuery, RepoProjectedPageIndexNodeResult,
    RepoProjectedPageIndexTreeQuery, RepoProjectedPageIndexTreeResult,
    RepoProjectedPageIndexTreeSearchQuery, RepoProjectedPageIndexTreeSearchResult,
    RepoProjectedPageIndexTreesQuery, RepoProjectedPageIndexTreesResult,
    RepoProjectedPageNavigationQuery, RepoProjectedPageNavigationResult,
    RepoProjectedPageNavigationSearchQuery, RepoProjectedPageNavigationSearchResult,
    RepoProjectedPageQuery, RepoProjectedPageResult, RepoProjectedPageSearchQuery,
    RepoProjectedPageSearchResult, RepoProjectedPagesQuery, RepoProjectedPagesResult,
    RepoProjectedRetrievalContextQuery, RepoProjectedRetrievalContextResult,
    RepoProjectedRetrievalHitQuery, RepoProjectedRetrievalHitResult, RepoProjectedRetrievalQuery,
    RepoProjectedRetrievalResult, RepositoryAnalysisOutput, build_repo_projected_gap_report,
    build_repo_projected_page, build_repo_projected_page_family_cluster,
    build_repo_projected_page_family_context, build_repo_projected_page_family_search,
    build_repo_projected_page_index_node, build_repo_projected_page_index_tree,
    build_repo_projected_page_index_tree_search, build_repo_projected_page_index_trees,
    build_repo_projected_page_navigation, build_repo_projected_page_navigation_search,
    build_repo_projected_pages, build_repo_projected_retrieval,
    build_repo_projected_retrieval_context, build_repo_projected_retrieval_hit,
};
use crate::analyzers::{
    RepositorySearchQueryCacheKey, load_cached_repository_search_result,
    store_cached_repository_search_result,
};
use crate::analyzers::{
    build_repo_projected_page_search_with_artifacts, repository_search_artifacts,
};
use crate::gateway::studio::router::handlers::repo::shared::{
    with_repo_analysis, with_repo_cached_analysis_bundle,
};
use crate::gateway::studio::router::{GatewayState, StudioApiError};
use crate::search::FuzzySearchOptions;

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

pub(crate) async fn run_repo_projected_pages(
    state: Arc<GatewayState>,
    query: RepoProjectedPagesQuery,
) -> Result<RepoProjectedPagesResult, StudioApiError> {
    run_repo_projected_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "REPO_PROJECTED_PAGES_PANIC",
        "Repo projected pages task failed unexpectedly",
        move |analysis| {
            Ok::<_, RepoIntelligenceError>(build_repo_projected_pages(&query, &analysis))
        },
    )
    .await
}

pub(crate) async fn run_repo_projected_gap_report(
    state: Arc<GatewayState>,
    query: RepoProjectedGapReportQuery,
) -> Result<RepoProjectedGapReportResult, StudioApiError> {
    run_repo_projected_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "REPO_PROJECTED_GAP_REPORT_PANIC",
        "Repo projected gap report task failed unexpectedly",
        move |analysis| {
            Ok::<_, RepoIntelligenceError>(build_repo_projected_gap_report(&query, &analysis))
        },
    )
    .await
}

pub(crate) async fn run_repo_projected_page(
    state: Arc<GatewayState>,
    query: RepoProjectedPageQuery,
) -> Result<RepoProjectedPageResult, StudioApiError> {
    run_repo_projected_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "REPO_PROJECTED_PAGE_PANIC",
        "Repo projected page task failed unexpectedly",
        move |analysis| build_repo_projected_page(&query, &analysis),
    )
    .await
}

pub(crate) async fn run_repo_projected_page_index_tree(
    state: Arc<GatewayState>,
    query: RepoProjectedPageIndexTreeQuery,
) -> Result<RepoProjectedPageIndexTreeResult, StudioApiError> {
    run_repo_projected_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "REPO_PROJECTED_PAGE_INDEX_TREE_PANIC",
        "Repo projected page-index tree task failed unexpectedly",
        move |analysis| build_repo_projected_page_index_tree(&query, &analysis),
    )
    .await
}

pub(crate) async fn run_repo_projected_page_index_node(
    state: Arc<GatewayState>,
    query: RepoProjectedPageIndexNodeQuery,
) -> Result<RepoProjectedPageIndexNodeResult, StudioApiError> {
    run_repo_projected_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "REPO_PROJECTED_PAGE_INDEX_NODE_PANIC",
        "Repo projected page-index node task failed unexpectedly",
        move |analysis| build_repo_projected_page_index_node(&query, &analysis),
    )
    .await
}

pub(crate) async fn run_repo_projected_page_index_trees(
    state: Arc<GatewayState>,
    query: RepoProjectedPageIndexTreesQuery,
) -> Result<RepoProjectedPageIndexTreesResult, StudioApiError> {
    run_repo_projected_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "REPO_PROJECTED_PAGE_INDEX_TREES_PANIC",
        "Repo projected page-index trees task failed unexpectedly",
        move |analysis| build_repo_projected_page_index_trees(&query, &analysis),
    )
    .await
}

pub(crate) async fn run_repo_projected_page_family_context(
    state: Arc<GatewayState>,
    query: RepoProjectedPageFamilyContextQuery,
) -> Result<RepoProjectedPageFamilyContextResult, StudioApiError> {
    run_repo_projected_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "REPO_PROJECTED_PAGE_FAMILY_CONTEXT_PANIC",
        "Repo projected page-family context task failed unexpectedly",
        move |analysis| build_repo_projected_page_family_context(&query, &analysis),
    )
    .await
}

pub(crate) async fn run_repo_projected_page_family_search(
    state: Arc<GatewayState>,
    query: RepoProjectedPageFamilySearchQuery,
) -> Result<RepoProjectedPageFamilySearchResult, StudioApiError> {
    run_repo_projected_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "REPO_PROJECTED_PAGE_FAMILY_SEARCH_PANIC",
        "Repo projected page-family search task failed unexpectedly",
        move |analysis| {
            Ok::<_, RepoIntelligenceError>(build_repo_projected_page_family_search(
                &query, &analysis,
            ))
        },
    )
    .await
}

pub(crate) async fn run_repo_projected_page_family_cluster(
    state: Arc<GatewayState>,
    query: RepoProjectedPageFamilyClusterQuery,
) -> Result<RepoProjectedPageFamilyClusterResult, StudioApiError> {
    run_repo_projected_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "REPO_PROJECTED_PAGE_FAMILY_CLUSTER_PANIC",
        "Repo projected page-family cluster task failed unexpectedly",
        move |analysis| build_repo_projected_page_family_cluster(&query, &analysis),
    )
    .await
}

pub(crate) async fn run_repo_projected_page_navigation(
    state: Arc<GatewayState>,
    query: RepoProjectedPageNavigationQuery,
) -> Result<RepoProjectedPageNavigationResult, StudioApiError> {
    run_repo_projected_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "REPO_PROJECTED_PAGE_NAVIGATION_PANIC",
        "Repo projected page navigation task failed unexpectedly",
        move |analysis| build_repo_projected_page_navigation(&query, &analysis),
    )
    .await
}

pub(crate) async fn run_repo_projected_page_navigation_search(
    state: Arc<GatewayState>,
    query: RepoProjectedPageNavigationSearchQuery,
) -> Result<RepoProjectedPageNavigationSearchResult, StudioApiError> {
    run_repo_projected_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "REPO_PROJECTED_PAGE_NAVIGATION_SEARCH_PANIC",
        "Repo projected page navigation search task failed unexpectedly",
        move |analysis| build_repo_projected_page_navigation_search(&query, &analysis),
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

async fn run_repo_projected_analysis<T, F>(
    state: Arc<GatewayState>,
    repo_id: String,
    panic_code: &'static str,
    panic_message: &'static str,
    build: F,
) -> Result<T, StudioApiError>
where
    T: Send + 'static,
    F: FnOnce(RepositoryAnalysisOutput) -> Result<T, RepoIntelligenceError> + Send + 'static,
{
    with_repo_analysis(state, repo_id, panic_code, panic_message, build).await
}
