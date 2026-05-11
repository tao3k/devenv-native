use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::{
    RepoProjectedPageFamilyClusterQuery, RepoProjectedPageFamilyClusterResult,
    RepoProjectedPageQuery,
};

use super::family_context::build_projected_page_family_clusters;
use super::lookup::build_projected_page;

/// Build one deterministic page-family cluster from stable projected identifiers.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page does
/// not exist for the analyzed repository, or
/// [`RepoIntelligenceError::UnknownProjectedPageFamilyCluster`] when the requested family is not
/// present for the projected page.
pub fn build_projected_page_family_cluster(
    query: &RepoProjectedPageFamilyClusterQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedPageFamilyClusterResult, RepoIntelligenceError> {
    let center_page = build_projected_page(
        &RepoProjectedPageQuery {
            repo_id: query.repo_id.clone(),
            page_id: query.page_id.clone(),
        },
        analysis,
    )?
    .page;

    let family = build_projected_page_family_clusters(&center_page, analysis, query.limit.max(1))
        .into_iter()
        .find(|family| family.kind == query.kind)
        .ok_or_else(
            || RepoIntelligenceError::UnknownProjectedPageFamilyCluster {
                repo_id: query.repo_id.clone().into(),
                page_id: query.page_id.clone().into(),
                kind: query.kind,
            },
        )?;

    Ok(RepoProjectedPageFamilyClusterResult {
        repo_id: query.repo_id.clone(),
        center_page,
        family,
    })
}
