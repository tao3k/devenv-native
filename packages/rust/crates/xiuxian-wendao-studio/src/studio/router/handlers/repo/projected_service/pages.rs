use std::sync::Arc;

use crate::studio::router::GatewayState;
use crate::studio::router::StudioApiError;
use xiuxian_wendao::analyzers::{
    RepoProjectedPageIndexNodeQuery, RepoProjectedPageIndexNodeResult,
    RepoProjectedPageIndexTreeQuery, RepoProjectedPageIndexTreeResult, RepoProjectedPageQuery,
    RepoProjectedPageResult, build_repo_projected_page, build_repo_projected_page_index_node,
    build_repo_projected_page_index_tree,
};

use super::analysis::run_repo_projected_analysis;

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
