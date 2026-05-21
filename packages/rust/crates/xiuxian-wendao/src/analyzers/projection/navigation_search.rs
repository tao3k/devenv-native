//! `analyzers::projection::navigation_search` owns Wendao analyzers projection navigation search behavior.

use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::projection::contracts::ProjectedPageRecord;
use crate::analyzers::projection::navigation_bundle::build_projected_page_navigation;
use crate::analyzers::projection::search::scored_projected_page_matches;
use crate::analyzers::{
    ProjectedPageNavigationSearchHit, RepoProjectedPageNavigationSearchQuery,
    RepoProjectedPageNavigationSearchResult,
};

/// Namespace boundary: this public name is scoped by its module owner.
/// Build repo projected-page navigation hits for a query string.
///
/// # Errors
///
/// Returns [`crate::analyzers::RepoIntelligenceError`] when building
/// an individual navigation bundle fails.
pub fn build_repo_projected_page_navigation_search(
    query: &RepoProjectedPageNavigationSearchQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedPageNavigationSearchResult, crate::analyzers::RepoIntelligenceError> {
    let normalized_query = query.query.trim().to_ascii_lowercase();
    let limit = query.limit.max(1);
    let mut matches =
        scored_projected_page_matches(normalized_query.as_str(), query.kind, analysis);

    matches.sort_by(
        |(left_score, left_page): &(u8, ProjectedPageRecord),
         (right_score, right_page): &(u8, ProjectedPageRecord)| {
            right_score
                .cmp(left_score)
                .then_with(|| left_page.title.cmp(&right_page.title))
                .then_with(|| left_page.page_id.cmp(&right_page.page_id))
        },
    );

    let mut hits = Vec::new();
    for (score, page) in matches.into_iter().take(limit) {
        let navigation = build_projected_page_navigation(
            &crate::analyzers::RepoProjectedPageNavigationQuery {
                repo_id: query.repo_id.clone(),
                page_id: page.page_id.clone(),
                node_id: None,
                family_kind: query.family_kind,
                related_limit: query.related_limit,
                family_limit: query.family_limit,
            },
            analysis,
        )?;

        hits.push(ProjectedPageNavigationSearchHit {
            search_score: score,
            navigation,
        });
    }

    Ok(RepoProjectedPageNavigationSearchResult {
        repo_id: query.repo_id.clone(),
        hits,
    })
}
