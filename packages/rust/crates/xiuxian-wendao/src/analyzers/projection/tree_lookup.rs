use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::{RepoProjectedPageIndexTreeQuery, RepoProjectedPageIndexTreeResult};

use super::markdown::build_projected_page_index_trees;

/// Resolve one projected page-index tree by projected page identifier.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the projected page cannot be
/// resolved into a page-index tree.
pub fn build_projected_page_index_tree(
    query: &RepoProjectedPageIndexTreeQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedPageIndexTreeResult, RepoIntelligenceError> {
    let tree = build_projected_page_index_trees(analysis)?
        .into_iter()
        .find(|tree| tree.page_id == query.page_id)
        .ok_or_else(|| RepoIntelligenceError::UnknownProjectedPage {
            repo_id: query.repo_id.clone(),
            page_id: query.page_id.clone(),
        })?;

    Ok(RepoProjectedPageIndexTreeResult {
        repo_id: query.repo_id.clone(),
        tree: Some(tree),
    })
}
