//! `analyzers::projection::retrieval_lookup` owns Wendao analyzers projection retrieval lookup behavior.

use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::projection::lookup::build_projected_page;
use crate::analyzers::projection::node_lookup::build_repo_projected_page_index_node;
use crate::analyzers::{
    ProjectedRetrievalHit, ProjectedRetrievalHitKind, RepoProjectedPageQuery,
    RepoProjectedRetrievalHitQuery, RepoProjectedRetrievalHitResult,
};

/// Build one retrieval hit for a projected page or page-index node.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the projected page or requested node
/// cannot be resolved.
pub fn build_projected_retrieval_hit(
    query: &RepoProjectedRetrievalHitQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedRetrievalHitResult, RepoIntelligenceError> {
    let page_record = build_projected_page(
        &RepoProjectedPageQuery {
            repo_id: query.repo_id.clone(),
            page_id: query.page_id.clone(),
        },
        analysis,
    )?
    .page;

    let node = if let Some(node_id) = &query.node_id {
        let node_result = build_repo_projected_page_index_node(
            &crate::analyzers::RepoProjectedPageIndexNodeQuery {
                repo_id: query.repo_id.clone(),
                page_id: query.page_id.clone(),
                node_id: node_id.clone(),
            },
            analysis,
        )?;
        node_result.hit
    } else {
        None
    };

    let kind = if node.is_some() {
        ProjectedRetrievalHitKind::PageIndexNode
    } else {
        ProjectedRetrievalHitKind::Page
    };

    Ok(RepoProjectedRetrievalHitResult {
        repo_id: query.repo_id.clone(),
        hit: ProjectedRetrievalHit {
            kind,
            page: page_record,
            node,
        },
    })
}
