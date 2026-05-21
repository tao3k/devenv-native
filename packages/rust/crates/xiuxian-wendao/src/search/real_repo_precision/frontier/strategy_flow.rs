//! Project backend frontier nodes into `SearchStrategyFlow` reasoning steps.

use crate::search::real_repo_precision::frontier::score::saturating_usize_to_u32;
use crate::search::real_repo_precision::{
    RealRepoKnowledgeScenario, RealRepoKnowledgeScenarioBackendFrontierNodeReceipt,
    RealRepoKnowledgeScenarioKind,
};

pub(super) fn attach_search_strategy_flow_projection(
    nodes: &mut [RealRepoKnowledgeScenarioBackendFrontierNodeReceipt],
) {
    for node in nodes.iter_mut() {
        let step_role = strategy_flow_step_role(node).to_string();
        let iteration_policy = strategy_flow_iteration_policy(node).to_string();
        let loop_candidate = strategy_flow_loop_candidate(node);
        let requires_llm_judgement = strategy_flow_requires_llm_judgement(node);
        node.strategy_flow_candidate_id = Some(format!("strategy-flow:candidate:{}", node.node_id));
        node.strategy_flow_transition_id = Some(format!(
            "strategy-flow:transition:{}:{}",
            node.node_id, node.backend_action
        ));
        node.strategy_flow_action = Some(node.backend_action.clone());
        node.strategy_flow_score_bps = Some(strategy_flow_score_bps(node));
        node.strategy_flow_step_role = Some(step_role);
        node.strategy_flow_iteration_policy = Some(iteration_policy);
        node.strategy_flow_loop_candidate = loop_candidate;
        node.strategy_flow_requires_llm_judgement = requires_llm_judgement;
    }

    let mut selected = nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| {
            node.backend_action != "prune" && !is_strategy_flow_validation_guard(node)
        })
        .map(|(index, node)| {
            (
                index,
                node.strategy_flow_score_bps.unwrap_or_default(),
                node.context_cost,
                node.node_id.clone(),
            )
        })
        .collect::<Vec<_>>();
    selected.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| left.2.cmp(&right.2))
            .then_with(|| left.3.cmp(&right.3))
    });

    for (rank, (index, _, _, _)) in selected.into_iter().enumerate() {
        let node = &mut nodes[index];
        node.strategy_flow_frontier_rank = Some(rank + 1);
        node.strategy_flow_context_budget_chars = Some(
            node.subagent_context_budget_chars
                .unwrap_or(node.context_cost),
        );
    }
}

pub(super) fn strategy_flow_intent_complexity_class(
    scenario: &RealRepoKnowledgeScenario,
) -> &'static str {
    let has_graph_hops = !scenario.required_relation_paths.is_empty()
        || !scenario.required_semantic_object_ids.is_empty();
    let has_guard = scenario.authority.is_some() || !scenario.forbidden_paths.is_empty();
    if has_graph_hops && has_guard {
        return "guarded_multi_hop";
    }
    if matches!(scenario.kind, RealRepoKnowledgeScenarioKind::AgentTask) {
        return "agentic";
    }
    if has_graph_hops {
        return "multi_hop_graph";
    }
    if matches!(
        scenario.kind,
        RealRepoKnowledgeScenarioKind::NaturalLanguageIntent
            | RealRepoKnowledgeScenarioKind::AmbiguousAlias
    ) {
        return "natural_language";
    }
    "known_item"
}

pub(super) fn strategy_flow_max_planned_depth(scenario: &RealRepoKnowledgeScenario) -> usize {
    if !scenario.required_relation_paths.is_empty()
        || !scenario.required_semantic_object_ids.is_empty()
    {
        3
    } else if matches!(
        scenario.kind,
        RealRepoKnowledgeScenarioKind::NaturalLanguageIntent
            | RealRepoKnowledgeScenarioKind::AmbiguousAlias
            | RealRepoKnowledgeScenarioKind::AgentTask
    ) {
        2
    } else {
        1
    }
}

pub(super) fn strategy_flow_loop_budget(
    scenario: &RealRepoKnowledgeScenario,
    cycle_candidate_node_count: usize,
) -> usize {
    if cycle_candidate_node_count == 0 {
        return 0;
    }
    if scenario.required_relation_paths.len() > 1
        || matches!(
            scenario.kind,
            RealRepoKnowledgeScenarioKind::AgentTask
                | RealRepoKnowledgeScenarioKind::AmbiguousAlias
        )
    {
        return 2;
    }
    1
}

pub(super) fn strategy_flow_refinement_topology(
    cycle_candidate_node_count: usize,
    llm_judgement_node_count: usize,
) -> &'static str {
    if cycle_candidate_node_count == 0 {
        return "acyclic_only";
    }
    if llm_judgement_node_count > 0 {
        return "cyclic_refinement_allowed";
    }
    "iterative_graph_refinement"
}

fn strategy_flow_step_role(
    node: &RealRepoKnowledgeScenarioBackendFrontierNodeReceipt,
) -> &'static str {
    match node.evidence_kind.as_str() {
        "anchor_query" => "intent_anchor",
        "relation_path" => "relation_refinement",
        "page_index_seed" => "page_index_grounding",
        "source_path" => "source_materialization",
        "authority_order" | "negative_guard" => "validation_guard",
        _ => "unknown",
    }
}

fn strategy_flow_iteration_policy(
    node: &RealRepoKnowledgeScenarioBackendFrontierNodeReceipt,
) -> &'static str {
    if node.backend_action == "prune" {
        return "closed";
    }
    match node.evidence_kind.as_str() {
        "anchor_query" => "expand_once",
        "relation_path" | "page_index_seed" => "can_revisit",
        "source_path" => "terminal_materialization",
        "authority_order" | "negative_guard" => "guard_only",
        _ => "single_pass",
    }
}

fn strategy_flow_loop_candidate(
    node: &RealRepoKnowledgeScenarioBackendFrontierNodeReceipt,
) -> bool {
    node.backend_action != "prune"
        && matches!(
            node.evidence_kind.as_str(),
            "relation_path" | "page_index_seed"
        )
}

fn strategy_flow_requires_llm_judgement(
    node: &RealRepoKnowledgeScenarioBackendFrontierNodeReceipt,
) -> bool {
    node.requires_subagent_judgement
}

fn is_strategy_flow_validation_guard(
    node: &RealRepoKnowledgeScenarioBackendFrontierNodeReceipt,
) -> bool {
    matches!(
        node.evidence_kind.as_str(),
        "authority_order" | "negative_guard"
    )
}

fn strategy_flow_score_bps(node: &RealRepoKnowledgeScenarioBackendFrontierNodeReceipt) -> u32 {
    if node.backend_action == "prune" {
        return 0;
    }
    let base = (node.graph_score_bps + node.authority_score_bps + node.coverage_score_bps) / 3;
    base.saturating_sub(saturating_usize_to_u32(node.context_cost).min(1_500))
}
