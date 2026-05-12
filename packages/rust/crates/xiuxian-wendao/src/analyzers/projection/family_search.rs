//! `analyzers::projection::family_search` owns Wendao analyzers projection family search behavior.

use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::projection::contracts::ProjectedPageRecord;
use crate::analyzers::projection::family_context::build_projected_page_family_context;
use crate::analyzers::projection::search::scored_projected_page_matches;
use crate::analyzers::{
    ProjectedPageFamilySearchHit, RepoProjectedPageFamilySearchQuery,
    RepoProjectedPageFamilySearchResult,
};

/// Namespace boundary: this public name is scoped by its module owner.
/// Build repo projected-page family search hits from scored projected pages.
#[must_use]
pub fn build_repo_projected_page_family_search(
    query: &RepoProjectedPageFamilySearchQuery,
    analysis: &RepositoryAnalysisOutput,
) -> RepoProjectedPageFamilySearchResult {
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

    RepoProjectedPageFamilySearchResult {
        repo_id: query.repo_id.clone(),
        hits: matches
            .into_iter()
            .take(limit)
            .filter_map(|(_, page)| {
                let context = build_projected_page_family_context(
                    &crate::analyzers::RepoProjectedPageFamilyContextQuery {
                        repo_id: query.repo_id.clone(),
                        page_id: page.page_id.clone(),
                        per_kind_limit: query.per_kind_limit,
                    },
                    analysis,
                )
                .ok()?;

                Some(ProjectedPageFamilySearchHit {
                    center_page: page,
                    families: context.families,
                })
            })
            .collect(),
    }
}
