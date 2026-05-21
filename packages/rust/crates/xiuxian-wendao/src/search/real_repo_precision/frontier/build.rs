//! Assemble the backend frontier receipt from reasoning-tree evidence.

use crate::search::real_repo_precision::{
    RealRepoKnowledgeScenario, RealRepoKnowledgeScenarioAuthorityReceipt,
    RealRepoKnowledgeScenarioBackendFrontierReceipt, RealRepoKnowledgeScenarioNegativeGuardReceipt,
    RealRepoKnowledgeScenarioReasoningTreeReceipt,
};

use super::counts::{
    backend_action_counts, julia_frontier_counts, strategy_flow_counts, subagent_frontier_counts,
};
use super::julia::{attach_julia_schedule_projection, julia_schedule_basis};
use super::nodes::{authority_node, negative_guard_node, node_from_reasoning_step};
use super::strategy_flow::{
    attach_search_strategy_flow_projection, strategy_flow_intent_complexity_class,
    strategy_flow_loop_budget, strategy_flow_max_planned_depth, strategy_flow_refinement_topology,
};

pub(crate) fn build_backend_frontier(
    scenario: &RealRepoKnowledgeScenario,
    reasoning_tree: &RealRepoKnowledgeScenarioReasoningTreeReceipt,
    authority: Option<&RealRepoKnowledgeScenarioAuthorityReceipt>,
    negative_guard: Option<&RealRepoKnowledgeScenarioNegativeGuardReceipt>,
) -> RealRepoKnowledgeScenarioBackendFrontierReceipt {
    let mut nodes = reasoning_tree
        .steps
        .iter()
        .map(|step| {
            node_from_reasoning_step(scenario, step, reasoning_tree, authority, negative_guard)
        })
        .collect::<Vec<_>>();

    if let Some(authority) = authority {
        nodes.push(authority_node(scenario, authority));
    }
    if let Some(negative_guard) = negative_guard {
        nodes.push(negative_guard_node(scenario, negative_guard));
    }
    attach_julia_schedule_projection(&mut nodes);
    attach_search_strategy_flow_projection(&mut nodes);

    let backend = backend_action_counts(&nodes);
    let subagent = subagent_frontier_counts(&nodes);
    let julia = julia_frontier_counts(&nodes);
    let strategy_flow = strategy_flow_counts(&nodes);
    let strategy_flow_loop_budget =
        strategy_flow_loop_budget(scenario, strategy_flow.cycle_candidate_node_count);
    let strategy_flow_refinement_topology = strategy_flow_refinement_topology(
        strategy_flow.cycle_candidate_node_count,
        strategy_flow.llm_judgement_node_count,
    );

    RealRepoKnowledgeScenarioBackendFrontierReceipt {
        strategy: "rust_controlled_backend_frontier_v1".to_string(),
        control_plane_owner: "rust".to_string(),
        graph_backend: "rust-baseline".to_string(),
        graph_backend_live: false,
        julia_schedule_basis: julia_schedule_basis().to_string(),
        node_count: nodes.len(),
        kept_node_count: backend.kept,
        pruned_node_count: backend.pruned,
        expand_node_count: backend.expand,
        subagent_judgement_node_count: subagent.judgement_node_count,
        subagent_fanout_group_count: subagent.fanout_group_count,
        subagent_fanout_node_count: subagent.fanout_node_count,
        subagent_max_parallel_width: subagent.max_parallel_width,
        subagent_context_budget_chars: subagent.context_budget_chars,
        julia_candidate_node_count: julia.candidates,
        julia_dispatch_node_count: julia.dispatches,
        julia_queue_node_count: julia.queued,
        julia_fallback_node_count: julia.fallbacks,
        julia_reject_node_count: julia.rejections,
        strategy_flow_projection_basis: "rust_receipt_projection_v1".to_string(),
        strategy_flow_candidate_node_count: strategy_flow.candidate_node_count,
        strategy_flow_transition_node_count: strategy_flow.transition_node_count,
        strategy_flow_frontier_node_count: strategy_flow.frontier_node_count,
        strategy_flow_context_budget_chars: strategy_flow.context_budget_chars,
        strategy_flow_intent_complexity_class: strategy_flow_intent_complexity_class(scenario)
            .to_string(),
        strategy_flow_initial_topology: "acyclic_evidence_dag".to_string(),
        strategy_flow_refinement_topology: strategy_flow_refinement_topology.to_string(),
        strategy_flow_max_planned_depth: strategy_flow_max_planned_depth(scenario),
        strategy_flow_loop_budget,
        strategy_flow_cycle_candidate_node_count: strategy_flow.cycle_candidate_node_count,
        strategy_flow_llm_judgement_node_count: strategy_flow.llm_judgement_node_count,
        selected_beam_width: backend.kept + backend.expand,
        nodes,
    }
}
