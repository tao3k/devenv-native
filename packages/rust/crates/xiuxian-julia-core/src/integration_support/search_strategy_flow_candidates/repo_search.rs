//! Repo-search candidate conversion for `SearchStrategyFlow`.

use super::discovery::{clamp_score, edge_kinds, line_context_cost, markdown_anchor};
use super::types::{SearchStrategyFlowCandidateInput, SearchStrategyFlowRepoSearchHit};
use xiuxian_wendao_runtime::transport::WENDAO_ARROW_FLIGHT_DATA_PLANE;

pub(crate) fn search_strategy_flow_candidate_input_from_repo_search_hit(
    hit: &SearchStrategyFlowRepoSearchHit<'_>,
) -> SearchStrategyFlowCandidateInput {
    let title = non_blank(hit.best_section)
        .or_else(|| non_blank(hit.title))
        .unwrap_or(hit.relative_path);
    let line_start = hit.line_start.unwrap_or(1).max(1);
    let line_end = hit.line_end.unwrap_or(line_start).max(line_start);
    let score = hit.score.unwrap_or(0.5).clamp(0.0, 1.0);

    SearchStrategyFlowCandidateInput {
        relative_path: hit.relative_path.to_owned(),
        heading_anchor: markdown_anchor(title),
        title: title.to_owned(),
        line_start,
        line_end,
        context_cost: line_context_cost(line_start, line_end),
        evidence_coverage: clamp_score(0.58 + (score * 0.34)),
        graph_score: clamp_score(0.62 + (score * 0.25)),
        authority_score: clamp_score(0.70 + (score * 0.12)),
        structural_score: clamp_score(0.66 + (score * 0.12)),
        uncertainty: clamp_score(0.34 - (score * 0.20)),
        blocked: false,
        edge_kinds: repo_search_edge_kinds(hit.relative_path, title),
    }
}

fn non_blank(value: Option<&str>) -> Option<&str> {
    value.and_then(|value| {
        let value = value.trim();
        if value.is_empty() { None } else { Some(value) }
    })
}

fn repo_search_edge_kinds(relative_path: &str, title: &str) -> Vec<String> {
    let mut kinds = edge_kinds(relative_path, title);
    kinds.push(WENDAO_ARROW_FLIGHT_DATA_PLANE.to_owned());
    kinds.push("repo-search".to_owned());
    kinds.sort();
    kinds.dedup();
    kinds
}
