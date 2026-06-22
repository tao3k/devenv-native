use std::collections::HashSet;

use serde_json::{Value, json};

use super::route::frontier_route_bucket;
use crate::integration_support::search_strategy_flow_flight::{
    SearchStrategyFlowCandidateId, SearchStrategyFlowFrontierRow, SearchStrategyFlowServiceResponse,
};

pub(super) fn validation_json(
    response: &SearchStrategyFlowServiceResponse,
    selected_candidate_ids: &HashSet<SearchStrategyFlowCandidateId>,
    query_understanding: &[Value],
    total_context: i64,
    selected_context: i64,
) -> Value {
    let required_evidence = required_evidence_values(query_understanding);
    let selected_required_evidence = selected_required_evidence_values(
        required_evidence.as_slice(),
        response.frontier.as_slice(),
    );
    let missing_required_evidence = required_evidence
        .iter()
        .filter(|evidence| !selected_required_evidence.contains(evidence))
        .cloned()
        .collect::<Vec<_>>();
    json!({
        "noVectorMode": response.candidates.iter().all(|row| row.semantic_score == 0.0),
        "materializedTopCandidate": response.planner_actions.iter().any(|row| {
            row.action_kind == "materialize" && selected_candidate_ids.contains(&row.candidate_id)
        }),
        "blockedEvidencePruned": response.candidates.iter().all(|row| {
            !row.blocked || !selected_candidate_ids.contains(&row.candidate_id)
        }),
        "selectedContextReduced": selected_context < total_context,
        "requiredEvidenceCovered": missing_required_evidence.is_empty(),
        "selectedRequiredEvidence": selected_required_evidence,
        "missingRequiredEvidence": missing_required_evidence,
    })
}

pub(super) fn selected_candidate_ids(
    rows: &[SearchStrategyFlowFrontierRow],
) -> HashSet<SearchStrategyFlowCandidateId> {
    rows.iter()
        .filter(|row| row.selected)
        .map(|row| row.candidate_id.clone())
        .collect()
}

fn required_evidence_values(query_understanding: &[Value]) -> Vec<String> {
    let mut values = query_understanding
        .iter()
        .filter_map(|row| {
            row.get("requiredEvidence")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
        })
        .map(str::to_owned)
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn selected_required_evidence_values(
    required_evidence: &[String],
    frontier: &[SearchStrategyFlowFrontierRow],
) -> Vec<String> {
    let selected_buckets = frontier
        .iter()
        .filter(|row| row.selected)
        .map(|row| frontier_route_bucket(row.candidate_id.as_str()))
        .collect::<HashSet<_>>();
    required_evidence
        .iter()
        .filter(|evidence| {
            selected_buckets.contains(required_evidence_route_bucket(evidence.as_str()))
        })
        .cloned()
        .collect()
}

fn required_evidence_route_bucket(evidence: &str) -> &str {
    match evidence {
        "ownership_boundary" => "authority",
        "validation_path" => "validation",
        "relation_path" => "link_graph",
        "page_index_seed" => "page_index",
        _ => evidence,
    }
}
