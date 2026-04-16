use std::sync::Arc;

use crate::analyzers::{
    RepoIntelligenceError, RepoProjectedGapReportQuery, RepoProjectedGapReportResult,
    RepoProjectedPageIndexTreesQuery, RepoProjectedPageIndexTreesResult, RepoProjectedPagesQuery,
    RepoProjectedPagesResult, build_repo_projected_gap_report,
    build_repo_projected_page_index_trees, build_repo_projected_pages,
};
use crate::gateway::studio::router::GatewayState;
use crate::gateway::studio::router::StudioApiError;

use super::analysis::run_repo_projected_analysis;

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
