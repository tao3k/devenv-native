//! `analyzers::projection::tree_search` owns Wendao analyzers projection tree search behavior.

use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::projection::contracts::{ProjectedPageIndexNode, ProjectionPageKind};
use crate::analyzers::{
    ProjectedPageIndexNodeHit, RepoProjectedPageIndexTreeSearchQuery,
    RepoProjectedPageIndexTreeSearchResult,
};

use super::markdown::build_projected_page_index_trees;

/// Namespace boundary: this public name is scoped by its module owner.
/// Search projected page-index nodes across projected page trees.
#[must_use]
pub fn build_repo_projected_page_index_tree_search(
    query: &RepoProjectedPageIndexTreeSearchQuery,
    analysis: &RepositoryAnalysisOutput,
) -> RepoProjectedPageIndexTreeSearchResult {
    let normalized_query = query.query.trim().to_ascii_lowercase();
    let limit = query.limit.max(1);
    let mut hits =
        scored_projected_page_index_node_hits(normalized_query.as_str(), query.kind, analysis);

    hits.sort_by(
        |(left_score, left_hit): &(u8, ProjectedPageIndexNodeHit),
         (right_score, right_hit): &(u8, ProjectedPageIndexNodeHit)| {
            right_score
                .cmp(left_score)
                .then_with(|| left_hit.page_title.cmp(&right_hit.page_title))
                .then_with(|| left_hit.node_title.cmp(&right_hit.node_title))
                .then_with(|| left_hit.node_id.cmp(&right_hit.node_id))
        },
    );

    RepoProjectedPageIndexTreeSearchResult {
        repo_id: query.repo_id.clone(),
        hits: hits.into_iter().take(limit).map(|(_, hit)| hit).collect(),
    }
}
/// `scored_projected_page_index_node_hits` public function boundary for Wendao.

#[must_use]
pub fn scored_projected_page_index_node_hits(
    query: &str,
    kind_filter: Option<ProjectionPageKind>,
    analysis: &RepositoryAnalysisOutput,
) -> Vec<(u8, ProjectedPageIndexNodeHit)> {
    let mut hits = Vec::new();
    let Ok(trees) = build_projected_page_index_trees(analysis) else {
        return hits;
    };
    for tree in trees {
        if let Some(kind) = kind_filter
            && tree.kind != kind
        {
            continue;
        }

        collect_matching_nodes(&tree, tree.roots.as_slice(), query, &mut hits);
    }
    hits
}

fn collect_matching_nodes(
    tree: &crate::analyzers::projection::contracts::ProjectedPageIndexTree,
    nodes: &[ProjectedPageIndexNode],
    query: &str,
    hits: &mut Vec<(u8, ProjectedPageIndexNodeHit)>,
) {
    for node in nodes {
        let score = node_match_score(node, query);
        if score > 0 {
            hits.push((
                score,
                ProjectedPageIndexNodeHit {
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
                },
            ));
        }
        collect_matching_nodes(tree, node.children.as_slice(), query, hits);
    }
}

fn node_match_score(node: &ProjectedPageIndexNode, query: &str) -> u8 {
    if query.is_empty() {
        return 0;
    }

    let title_lc = node.title.to_ascii_lowercase();
    if title_lc == query {
        return 100;
    }
    if title_lc.starts_with(query) {
        return 90;
    }
    if title_lc.contains(query) {
        return 75;
    }
    if node
        .structural_path
        .iter()
        .any(|segment| segment.to_ascii_lowercase().contains(query))
    {
        return 60;
    }
    if node.text.to_ascii_lowercase().contains(query) {
        return 45;
    }

    0
}
