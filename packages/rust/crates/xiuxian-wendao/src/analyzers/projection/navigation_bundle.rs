use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::{
    RepoProjectedPageFamilyClusterQuery, RepoProjectedPageIndexTreeQuery,
    RepoProjectedPageNavigationQuery, RepoProjectedPageNavigationResult,
    RepoProjectedRetrievalContextQuery,
};

use super::family_lookup::build_projected_page_family_cluster;
use super::retrieval_context::build_projected_retrieval_context;
use super::tree_lookup::build_projected_page_index_tree;

/// Build one deterministic page-centric Stage-2 navigation bundle.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page is
/// not present in the analysis output, [`RepoIntelligenceError::UnknownProjectedPageIndexNode`]
/// when the requested projected page-index node is not present for the projected page, or
/// [`RepoIntelligenceError::UnknownProjectedPageFamilyCluster`] when the requested family is not
/// present for the projected page.
pub fn build_projected_page_navigation(
    query: &RepoProjectedPageNavigationQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedPageNavigationResult, RepoIntelligenceError> {
    let context = build_projected_retrieval_context(
        &RepoProjectedRetrievalContextQuery {
            repo_id: query.repo_id.clone(),
            page_id: query.page_id.clone(),
            node_id: query.node_id.clone(),
            related_limit: query.related_limit,
        },
        analysis,
    )?;
    let tree = build_projected_page_index_tree(
        &RepoProjectedPageIndexTreeQuery {
            repo_id: query.repo_id.clone(),
            page_id: query.page_id.clone(),
        },
        analysis,
    )?
    .tree;
    let family_cluster = query
        .family_kind
        .map(|kind| {
            build_projected_page_family_cluster(
                &RepoProjectedPageFamilyClusterQuery {
                    repo_id: query.repo_id.clone(),
                    page_id: query.page_id.clone(),
                    kind,
                    limit: query.family_limit,
                },
                analysis,
            )
            .map(|result| result.family)
        })
        .transpose()?;

    Ok(RepoProjectedPageNavigationResult {
        repo_id: query.repo_id.clone(),
        center: Some(context.center),
        related_pages: context.related_pages,
        node_context: context.node_context,
        tree,
        family_cluster,
    })
}
