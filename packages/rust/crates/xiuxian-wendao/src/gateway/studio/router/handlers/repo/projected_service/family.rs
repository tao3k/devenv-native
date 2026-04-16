use std::sync::Arc;

use crate::analyzers::{
    RepoIntelligenceError, RepoProjectedPageFamilyClusterQuery,
    RepoProjectedPageFamilyClusterResult, RepoProjectedPageFamilyContextQuery,
    RepoProjectedPageFamilyContextResult, RepoProjectedPageFamilySearchQuery,
    RepoProjectedPageFamilySearchResult, RepoProjectedPageNavigationQuery,
    RepoProjectedPageNavigationResult, RepoProjectedPageNavigationSearchQuery,
    RepoProjectedPageNavigationSearchResult, build_repo_projected_page_family_cluster,
    build_repo_projected_page_family_context, build_repo_projected_page_family_search,
    build_repo_projected_page_navigation, build_repo_projected_page_navigation_search,
};
use crate::gateway::studio::router::GatewayState;
use crate::gateway::studio::router::StudioApiError;

use super::analysis::run_repo_projected_analysis;

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
