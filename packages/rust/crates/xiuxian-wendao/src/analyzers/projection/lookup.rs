use crate::analyzers::{
    RepoIntelligenceError, RepoProjectedPageQuery, RepoProjectedPageResult,
    RepositoryAnalysisOutput,
};

use super::pages::build_projected_pages;

/// Resolve one deterministic projected page by stable page identifier.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page does
/// not exist for the analyzed repository.
pub fn build_projected_page(
    query: &RepoProjectedPageQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedPageResult, RepoIntelligenceError> {
    let page = build_projected_pages(analysis)
        .into_iter()
        .find(|page| page.page_id == query.page_id)
        .ok_or_else(|| RepoIntelligenceError::UnknownProjectedPage {
            repo_id: query.repo_id.clone().into(),
            page_id: query.page_id.clone().into(),
        })?;

    Ok(RepoProjectedPageResult {
        repo_id: query.repo_id.clone(),
        page,
    })
}
