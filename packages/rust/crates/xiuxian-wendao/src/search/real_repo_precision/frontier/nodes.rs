//! Materialize backend frontier nodes from scenario reasoning-tree evidence.

use crate::search::real_repo_precision::frontier::score::{
    authority_score_for_step, backend_action_for_step, evidence_kind_for_step,
    graph_score_for_step, node_id, parent_node_id, step_context_cost,
    subagent_context_budget_chars, subagent_priority_score_bps,
};
use crate::search::real_repo_precision::{
    RealRepoKnowledgeScenario, RealRepoKnowledgeScenarioAuthorityReceipt,
    RealRepoKnowledgeScenarioBackendFrontierNodeReceipt,
    RealRepoKnowledgeScenarioNegativeGuardReceipt, RealRepoKnowledgeScenarioReasoningTreeReceipt,
    RealRepoKnowledgeScenarioReasoningTreeStepReceipt,
};

pub(super) fn node_from_reasoning_step(
    scenario: &RealRepoKnowledgeScenario,
    step: &RealRepoKnowledgeScenarioReasoningTreeStepReceipt,
    reasoning_tree: &RealRepoKnowledgeScenarioReasoningTreeReceipt,
    authority: Option<&RealRepoKnowledgeScenarioAuthorityReceipt>,
    negative_guard: Option<&RealRepoKnowledgeScenarioNegativeGuardReceipt>,
) -> RealRepoKnowledgeScenarioBackendFrontierNodeReceipt {
    let evidence_kind = evidence_kind_for_step(step);
    let negative_guard_hit = step.path.as_ref().is_some_and(|path| {
        negative_guard.is_some_and(|guard| {
            guard
                .matched_forbidden_paths
                .iter()
                .any(|item| item == path)
        })
    });
    let backend_action = backend_action_for_step(step, negative_guard_hit);
    let requires_subagent_judgement = backend_action == "expand";
    let subagent_priority_score_bps = requires_subagent_judgement
        .then(|| subagent_priority_score_bps(step, authority, negative_guard_hit));
    let subagent_context_budget_chars =
        requires_subagent_judgement.then(|| subagent_context_budget_chars(step));
    RealRepoKnowledgeScenarioBackendFrontierNodeReceipt {
        node_id: node_id(scenario, step.step_index),
        parent_node_id: parent_node_id(scenario, step, reasoning_tree),
        reasoning_step_index: Some(step.step_index),
        step_kind: step.step_kind.clone(),
        evidence_kind: evidence_kind.to_string(),
        evidence_id: step.evidence_id.clone(),
        query_id: step.query_id.clone(),
        path: step.path.clone(),
        relation: step.relation.clone(),
        semantic_object_id: step.semantic_object_id.clone(),
        disclosure_depth: step.disclosure_depth,
        parallel_group: format!("scenario:{}:depth:{}", scenario.id, step.disclosure_depth),
        graph_batch_key: format!("{}:{}", scenario.kind.as_str(), step.step_kind),
        graph_score_bps: graph_score_for_step(step),
        authority_score_bps: authority_score_for_step(step, authority),
        coverage_score_bps: if step.passed { 10_000 } else { 0 },
        context_cost: step_context_cost(step),
        backend_action,
        requires_subagent_judgement,
        subagent_prompt_hint: requires_subagent_judgement
            .then(|| "judge whether this frontier branch should expand".to_string()),
        subagent_fanout_group_id: requires_subagent_judgement
            .then(|| format!("subagent:{}:depth:{}", scenario.id, step.disclosure_depth)),
        subagent_judgement_kind: requires_subagent_judgement
            .then(|| "branch_expand_candidate".to_string()),
        subagent_priority_score_bps,
        subagent_context_budget_chars,
        julia_algorithm_id: None,
        julia_profile_id: None,
        julia_capability: None,
        julia_schedule_action: None,
        julia_schedule_reason: None,
        julia_schedule_confidence_score: None,
        julia_selected_batch_size: None,
        strategy_flow_candidate_id: None,
        strategy_flow_transition_id: None,
        strategy_flow_action: None,
        strategy_flow_score_bps: None,
        strategy_flow_frontier_rank: None,
        strategy_flow_context_budget_chars: None,
        strategy_flow_step_role: None,
        strategy_flow_iteration_policy: None,
        strategy_flow_loop_candidate: false,
        strategy_flow_requires_llm_judgement: false,
    }
}

pub(super) fn authority_node(
    scenario: &RealRepoKnowledgeScenario,
    authority: &RealRepoKnowledgeScenarioAuthorityReceipt,
) -> RealRepoKnowledgeScenarioBackendFrontierNodeReceipt {
    RealRepoKnowledgeScenarioBackendFrontierNodeReceipt {
        node_id: format!("frontier:{}:authority", scenario.id),
        parent_node_id: None,
        reasoning_step_index: None,
        step_kind: "authority_order".to_string(),
        evidence_kind: "authority_order".to_string(),
        evidence_id: format!("authority:{}", authority.preferred_path),
        query_id: None,
        path: Some(authority.preferred_path.clone()),
        relation: None,
        semantic_object_id: None,
        disclosure_depth: 1,
        parallel_group: format!("scenario:{}:depth:1", scenario.id),
        graph_batch_key: format!("{}:authority_order", scenario.kind.as_str()),
        graph_score_bps: if authority.passed { 9_000 } else { 2_000 },
        authority_score_bps: if authority.passed { 10_000 } else { 0 },
        coverage_score_bps: if authority.passed { 10_000 } else { 0 },
        context_cost: authority.preferred_path.len()
            + authority
                .competing_paths
                .iter()
                .map(String::len)
                .sum::<usize>(),
        backend_action: if authority.passed { "keep" } else { "prune" }.to_string(),
        requires_subagent_judgement: false,
        subagent_prompt_hint: None,
        subagent_fanout_group_id: None,
        subagent_judgement_kind: None,
        subagent_priority_score_bps: None,
        subagent_context_budget_chars: None,
        julia_algorithm_id: None,
        julia_profile_id: None,
        julia_capability: None,
        julia_schedule_action: None,
        julia_schedule_reason: None,
        julia_schedule_confidence_score: None,
        julia_selected_batch_size: None,
        strategy_flow_candidate_id: None,
        strategy_flow_transition_id: None,
        strategy_flow_action: None,
        strategy_flow_score_bps: None,
        strategy_flow_frontier_rank: None,
        strategy_flow_context_budget_chars: None,
        strategy_flow_step_role: None,
        strategy_flow_iteration_policy: None,
        strategy_flow_loop_candidate: false,
        strategy_flow_requires_llm_judgement: false,
    }
}

pub(super) fn negative_guard_node(
    scenario: &RealRepoKnowledgeScenario,
    negative_guard: &RealRepoKnowledgeScenarioNegativeGuardReceipt,
) -> RealRepoKnowledgeScenarioBackendFrontierNodeReceipt {
    RealRepoKnowledgeScenarioBackendFrontierNodeReceipt {
        node_id: format!("frontier:{}:negative-guard", scenario.id),
        parent_node_id: None,
        reasoning_step_index: None,
        step_kind: "negative_guard".to_string(),
        evidence_kind: "negative_guard".to_string(),
        evidence_id: format!("negative-guard:{}", scenario.id),
        query_id: None,
        path: None,
        relation: None,
        semantic_object_id: None,
        disclosure_depth: 1,
        parallel_group: format!("scenario:{}:depth:1", scenario.id),
        graph_batch_key: format!("{}:negative_guard", scenario.kind.as_str()),
        graph_score_bps: if negative_guard.passed { 9_000 } else { 0 },
        authority_score_bps: 10_000,
        coverage_score_bps: if negative_guard.passed { 10_000 } else { 0 },
        context_cost: negative_guard.forbidden_paths.iter().map(String::len).sum(),
        backend_action: if negative_guard.passed {
            "keep"
        } else {
            "prune"
        }
        .to_string(),
        requires_subagent_judgement: false,
        subagent_prompt_hint: None,
        subagent_fanout_group_id: None,
        subagent_judgement_kind: None,
        subagent_priority_score_bps: None,
        subagent_context_budget_chars: None,
        julia_algorithm_id: None,
        julia_profile_id: None,
        julia_capability: None,
        julia_schedule_action: None,
        julia_schedule_reason: None,
        julia_schedule_confidence_score: None,
        julia_selected_batch_size: None,
        strategy_flow_candidate_id: None,
        strategy_flow_transition_id: None,
        strategy_flow_action: None,
        strategy_flow_score_bps: None,
        strategy_flow_frontier_rank: None,
        strategy_flow_context_budget_chars: None,
        strategy_flow_step_role: None,
        strategy_flow_iteration_policy: None,
        strategy_flow_loop_candidate: false,
        strategy_flow_requires_llm_judgement: false,
    }
}
