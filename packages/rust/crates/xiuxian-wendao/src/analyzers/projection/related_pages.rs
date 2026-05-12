//! `analyzers::projection::related_pages` owns Wendao analyzers projection related pages behavior.

use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::projection::contracts::{ProjectedPageRecord, ProjectionPageKind};

use super::pages::build_projected_pages;
/// `PROJECTION_PAGE_KIND_ORDER` public constant boundary for Wendao.

pub const PROJECTION_PAGE_KIND_ORDER: [ProjectionPageKind; 4] = [
    ProjectionPageKind::Reference,
    ProjectionPageKind::HowTo,
    ProjectionPageKind::Tutorial,
    ProjectionPageKind::Explanation,
];
/// `scored_related_projected_pages` public function boundary for Wendao.

#[must_use]
pub fn scored_related_projected_pages(
    page: &ProjectedPageRecord,
    analysis: &RepositoryAnalysisOutput,
) -> Vec<(usize, ProjectedPageRecord)> {
    let mut matches = Vec::new();

    for candidate in build_projected_pages(analysis) {
        if candidate.page_id == page.page_id {
            continue;
        }

        let score = calculate_relation_score(page, &candidate);
        if score > 0 {
            matches.push((score, candidate));
        }
    }

    matches.sort_by(
        |(left_score, left_page): &(usize, ProjectedPageRecord),
         (right_score, right_page): &(usize, ProjectedPageRecord)| {
            right_score
                .cmp(left_score)
                .then_with(|| left_page.title.cmp(&right_page.title))
                .then_with(|| left_page.page_id.cmp(&right_page.page_id))
        },
    );

    matches
}
/// `find_related_pages` public function boundary for Wendao.

#[must_use]
pub fn find_related_pages(
    page: &ProjectedPageRecord,
    analysis: &RepositoryAnalysisOutput,
    limit: usize,
) -> Vec<ProjectedPageRecord> {
    scored_related_projected_pages(page, analysis)
        .into_iter()
        .take(limit)
        .map(|(_, page)| page)
        .collect()
}

fn calculate_relation_score(page: &ProjectedPageRecord, candidate: &ProjectedPageRecord) -> usize {
    shared_count(page.module_ids.as_slice(), candidate.module_ids.as_slice())
        + shared_count(page.symbol_ids.as_slice(), candidate.symbol_ids.as_slice())
        + shared_count(page.doc_ids.as_slice(), candidate.doc_ids.as_slice())
        + shared_count(
            page.example_ids.as_slice(),
            candidate.example_ids.as_slice(),
        )
}

fn shared_count(left: &[String], right: &[String]) -> usize {
    left.iter()
        .filter(|item| right.iter().any(|candidate| candidate == *item))
        .count()
}
