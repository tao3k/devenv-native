//! Score frontier evidence and derive stable reasoning-tree node identifiers.

use crate::search::real_repo_precision::{
    RealRepoKnowledgeScenario, RealRepoKnowledgeScenarioAuthorityReceipt,
    RealRepoKnowledgeScenarioReasoningTreeReceipt,
    RealRepoKnowledgeScenarioReasoningTreeStepReceipt,
    RealRepoMarkdownKnowledgeSemanticRelationPathReceipt,
};

pub(super) fn saturating_usize_to_u32(value: usize) -> u32 {
    value.try_into().unwrap_or(u32::MAX)
}

pub(super) fn evidence_kind_for_step(
    step: &RealRepoKnowledgeScenarioReasoningTreeStepReceipt,
) -> &'static str {
    match step.step_kind.as_str() {
        "anchor_query" => "anchor_query",
        "semantic_relation" => "relation_path",
        "page_index_seed" => "page_index_seed",
        "source_evidence" => "source_path",
        _ => "unknown",
    }
}

pub(super) fn backend_action_for_step(
    step: &RealRepoKnowledgeScenarioReasoningTreeStepReceipt,
    negative_guard_hit: bool,
) -> String {
    if !step.passed || negative_guard_hit {
        return "prune".to_string();
    }
    if step.step_kind == "anchor_query" {
        return "expand".to_string();
    }
    "keep".to_string()
}

pub(super) fn subagent_priority_score_bps(
    step: &RealRepoKnowledgeScenarioReasoningTreeStepReceipt,
    authority: Option<&RealRepoKnowledgeScenarioAuthorityReceipt>,
    negative_guard_hit: bool,
) -> u32 {
    if negative_guard_hit || !step.passed {
        return 0;
    }
    let graph_score = graph_score_for_step(step);
    let authority_score = authority_score_for_step(step, authority);
    let coverage_score = 10_000;
    let context_penalty = saturating_usize_to_u32(step_context_cost(step)).min(1_500);
    ((graph_score + authority_score + coverage_score) / 3).saturating_sub(context_penalty)
}

pub(super) fn subagent_context_budget_chars(
    step: &RealRepoKnowledgeScenarioReasoningTreeStepReceipt,
) -> usize {
    let base = 512 + step_context_cost(step).saturating_mul(2);
    base.clamp(640, 1_600)
}

pub(super) fn graph_score_for_step(
    step: &RealRepoKnowledgeScenarioReasoningTreeStepReceipt,
) -> u32 {
    if !step.passed {
        return 0;
    }
    let base: u32 = match step.step_kind.as_str() {
        "anchor_query" => 7_000,
        "semantic_relation" => 8_500,
        "page_index_seed" => 8_000,
        "source_evidence" => 9_000,
        _ => 5_000,
    };
    let rank_penalty =
        saturating_usize_to_u32(step.zero_based_rank.unwrap_or_default().min(10)) * 250;
    base.saturating_sub(rank_penalty)
}

pub(super) fn authority_score_for_step(
    step: &RealRepoKnowledgeScenarioReasoningTreeStepReceipt,
    authority: Option<&RealRepoKnowledgeScenarioAuthorityReceipt>,
) -> u32 {
    let Some(authority) = authority else {
        return 7_000;
    };
    let Some(path) = step.path.as_ref() else {
        return 7_000;
    };
    if path == &authority.preferred_path {
        return 10_000;
    }
    if authority.competing_paths.iter().any(|item| item == path) {
        return 3_000;
    }
    7_000
}

pub(super) fn parent_node_id(
    scenario: &RealRepoKnowledgeScenario,
    step: &RealRepoKnowledgeScenarioReasoningTreeStepReceipt,
    reasoning_tree: &RealRepoKnowledgeScenarioReasoningTreeReceipt,
) -> Option<String> {
    if step.disclosure_depth == 0 {
        return None;
    }
    reasoning_tree
        .steps
        .iter()
        .rev()
        .find(|candidate| {
            candidate.step_index < step.step_index
                && candidate.disclosure_depth < step.disclosure_depth
        })
        .map(|candidate| node_id(scenario, candidate.step_index))
}

pub(super) fn node_id(scenario: &RealRepoKnowledgeScenario, step_index: usize) -> String {
    format!("frontier:{}:step:{step_index}", scenario.id)
}

pub(super) fn step_context_cost(step: &RealRepoKnowledgeScenarioReasoningTreeStepReceipt) -> usize {
    let relation_cost = step.relation.as_ref().map_or(0, relation_context_cost);
    step.evidence_id.len()
        + step.query_id.as_ref().map_or(0, String::len)
        + step.path.as_ref().map_or(0, String::len)
        + step.semantic_object_id.as_ref().map_or(0, String::len)
        + relation_cost
}

fn relation_context_cost(relation: &RealRepoMarkdownKnowledgeSemanticRelationPathReceipt) -> usize {
    relation.source.len() + relation.kind.len() + relation.target.len()
}
