use crate::integration_support::search_strategy_flow_candidates::SearchStrategyFlowCandidateInput;

pub(super) fn merge_candidate_discovery_result(
    candidates: &mut Vec<SearchStrategyFlowCandidateInput>,
    candidate: SearchStrategyFlowCandidateInput,
) {
    let Some(existing) = candidates.iter_mut().find(|existing| {
        existing.relative_path == candidate.relative_path
            && existing.heading_anchor == candidate.heading_anchor
    }) else {
        candidates.push(candidate);
        return;
    };

    existing.evidence_coverage = existing.evidence_coverage.max(candidate.evidence_coverage);
    existing.graph_score = existing.graph_score.max(candidate.graph_score);
    existing.authority_score = existing.authority_score.max(candidate.authority_score);
    existing.structural_score = existing.structural_score.max(candidate.structural_score);
    existing.uncertainty = existing.uncertainty.min(candidate.uncertainty);
    existing.context_cost = existing.context_cost.min(candidate.context_cost);
    existing.line_start = existing.line_start.min(candidate.line_start);
    existing.line_end = existing.line_end.max(candidate.line_end);
    existing.edge_kinds.extend(candidate.edge_kinds);
    existing.edge_kinds.sort();
    existing.edge_kinds.dedup();
}
