use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::projection::contracts::ProjectedPageIndexNode;
use crate::analyzers::projection::tree_lookup::build_projected_page_index_tree;
use crate::analyzers::{
    ProjectedPageIndexNodeHit, RepoProjectedPageIndexNodeQuery, RepoProjectedPageIndexNodeResult,
};

/// Resolve one projected page-index node hit by page and node identifier.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the projected page or node cannot be
/// resolved for the requested repository.
pub fn build_repo_projected_page_index_node(
    query: &RepoProjectedPageIndexNodeQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedPageIndexNodeResult, RepoIntelligenceError> {
    let tree_result = build_projected_page_index_tree(
        &crate::analyzers::RepoProjectedPageIndexTreeQuery {
            repo_id: query.repo_id.clone(),
            page_id: query.page_id.clone(),
        },
        analysis,
    )?;

    let tree = tree_result
        .tree
        .ok_or_else(|| RepoIntelligenceError::UnknownProjectedPage {
            repo_id: query.repo_id.clone().into(),
            page_id: query.page_id.clone().into(),
        })?;

    let hit = find_node(tree.roots.as_slice(), query.node_id.as_str())
        .map(|node| ProjectedPageIndexNodeHit {
            repo_id: tree.repo_id.clone(),
            page_id: tree.page_id.clone(),
            page_title: tree.title.clone(),
            page_kind: tree.kind,
            path: tree.path.clone(),
            doc_id: tree.doc_id.clone(),
            node_id: node.node_id.clone(),
            node_title: node.title.clone(),
            structural_path: node.structural_path.clone(),
            line_range: node.line_range,
            text: node.text.clone(),
        })
        .ok_or_else(|| RepoIntelligenceError::UnknownProjectedPageIndexNode {
            repo_id: query.repo_id.clone().into(),
            page_id: query.page_id.clone().into(),
            node_id: query.node_id.clone().into(),
        })?;

    Ok(RepoProjectedPageIndexNodeResult {
        repo_id: query.repo_id.clone(),
        hit: Some(hit),
    })
}

fn find_node<'a>(
    nodes: &'a [ProjectedPageIndexNode],
    node_id: &str,
) -> Option<&'a ProjectedPageIndexNode> {
    for node in nodes {
        if node.node_id == node_id {
            return Some(node);
        }
        if let Some(found) = find_node(node.children.as_slice(), node_id) {
            return Some(found);
        }
    }
    None
}
