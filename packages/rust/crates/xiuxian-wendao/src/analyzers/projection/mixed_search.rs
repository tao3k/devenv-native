//! `analyzers::projection::mixed_search` owns Wendao analyzers projection mixed search behavior.

use std::collections::BTreeMap;

use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::{
    ProjectedRetrievalHit, ProjectedRetrievalHitKind, RepoProjectedRetrievalQuery,
    RepoProjectedRetrievalResult,
};

use super::pages::build_projected_pages;
use super::search::scored_projected_page_matches;
use super::tree_search::scored_projected_page_index_node_hits;

/// Build deterministic mixed retrieval hits from projected pages and projected page-index trees.
#[must_use]
pub fn build_projected_retrieval(
    query: &RepoProjectedRetrievalQuery,
    analysis: &RepositoryAnalysisOutput,
) -> RepoProjectedRetrievalResult {
    let normalized_query = query.query.trim().to_ascii_lowercase();
    let limit = query.limit.max(1);
    let pages = build_projected_pages(analysis)
        .into_iter()
        .map(|page| (page.page_id.clone(), page))
        .collect::<BTreeMap<_, _>>();
    let mut hits = Vec::new();

    for (score, page) in
        scored_projected_page_matches(normalized_query.as_str(), query.kind, analysis)
    {
        hits.push((
            score,
            ProjectedRetrievalHit {
                kind: ProjectedRetrievalHitKind::Page,
                page,
                node: None,
            },
        ));
    }

    for (score, node) in
        scored_projected_page_index_node_hits(normalized_query.as_str(), query.kind, analysis)
    {
        let Some(page) = pages.get(&node.page_id).cloned() else {
            continue;
        };
        hits.push((
            score,
            ProjectedRetrievalHit {
                kind: ProjectedRetrievalHitKind::PageIndexNode,
                page,
                node: Some(node),
            },
        ));
    }

    hits.sort_by(
        |(left_score, left_hit): &(u8, ProjectedRetrievalHit),
         (right_score, right_hit): &(u8, ProjectedRetrievalHit)| {
            right_score
                .cmp(left_score)
                .then_with(|| left_hit.page.title.cmp(&right_hit.page.title))
                .then_with(|| retrieval_hit_title(left_hit).cmp(retrieval_hit_title(right_hit)))
                .then_with(|| left_hit.kind.cmp(&right_hit.kind))
                .then_with(|| left_hit.page.page_id.cmp(&right_hit.page.page_id))
                .then_with(|| retrieval_hit_node_id(left_hit).cmp(retrieval_hit_node_id(right_hit)))
        },
    );

    RepoProjectedRetrievalResult {
        repo_id: query.repo_id.clone(),
        hits: hits.into_iter().take(limit).map(|(_, hit)| hit).collect(),
    }
}

fn retrieval_hit_title(hit: &ProjectedRetrievalHit) -> &str {
    hit.node
        .as_ref()
        .map_or(hit.page.title.as_str(), |node| node.node_title.as_str())
}

fn retrieval_hit_node_id(hit: &ProjectedRetrievalHit) -> &str {
    hit.node.as_ref().map_or("", |node| node.node_id.as_str())
}
