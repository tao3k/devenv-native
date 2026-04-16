use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::{
    ProjectedPageFamilyCluster, ProjectedPageFamilyContextEntry,
    RepoProjectedPageFamilyContextQuery, RepoProjectedPageFamilyContextResult,
    RepoProjectedPageQuery,
};

use super::lookup::build_projected_page;
use super::related_pages::{PROJECTION_PAGE_KIND_ORDER, scored_related_projected_pages};

/// Build deterministic page-family context around one stable projected page.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page is
/// not present in the analysis output.
pub fn build_projected_page_family_context(
    query: &RepoProjectedPageFamilyContextQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedPageFamilyContextResult, RepoIntelligenceError> {
    let center_page = build_projected_page(
        &RepoProjectedPageQuery {
            repo_id: query.repo_id.clone(),
            page_id: query.page_id.clone(),
        },
        analysis,
    )?
    .page;

    Ok(RepoProjectedPageFamilyContextResult {
        repo_id: query.repo_id.clone(),
        families: build_projected_page_family_clusters(
            &center_page,
            analysis,
            query.per_kind_limit,
        ),
        center_page,
    })
}

#[must_use]
pub(crate) fn build_projected_page_family_clusters(
    center_page: &crate::analyzers::projection::ProjectedPageRecord,
    analysis: &RepositoryAnalysisOutput,
    per_kind_limit: usize,
) -> Vec<ProjectedPageFamilyCluster> {
    let related_pages = scored_related_projected_pages(center_page, analysis);
    PROJECTION_PAGE_KIND_ORDER
        .into_iter()
        .filter_map(|kind| {
            let pages = related_pages
                .iter()
                .filter(|(_, page)| page.kind == kind)
                .take(per_kind_limit)
                .map(
                    |(shared_anchor_score, page)| ProjectedPageFamilyContextEntry {
                        shared_anchor_score: *shared_anchor_score,
                        page: page.clone(),
                    },
                )
                .collect::<Vec<_>>();
            (!pages.is_empty()).then_some(ProjectedPageFamilyCluster { kind, pages })
        })
        .collect()
}
