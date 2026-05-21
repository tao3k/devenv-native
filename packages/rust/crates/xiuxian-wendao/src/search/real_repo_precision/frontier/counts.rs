//! Count frontier-node categories for the real-repository precision receipt.

use crate::search::real_repo_precision::RealRepoKnowledgeScenarioBackendFrontierNodeReceipt;

pub(super) struct BackendActionCounts {
    pub(super) kept: usize,
    pub(super) pruned: usize,
    pub(super) expand: usize,
}

pub(super) fn backend_action_counts(
    nodes: &[RealRepoKnowledgeScenarioBackendFrontierNodeReceipt],
) -> BackendActionCounts {
    BackendActionCounts {
        kept: nodes
            .iter()
            .filter(|node| node.backend_action == "keep")
            .count(),
        pruned: nodes
            .iter()
            .filter(|node| node.backend_action == "prune")
            .count(),
        expand: nodes
            .iter()
            .filter(|node| node.backend_action == "expand")
            .count(),
    }
}

pub(super) struct SubagentFrontierCounts {
    pub(super) judgement_node_count: usize,
    pub(super) fanout_node_count: usize,
    pub(super) fanout_group_count: usize,
    pub(super) max_parallel_width: usize,
    pub(super) context_budget_chars: usize,
}

pub(super) fn subagent_frontier_counts(
    nodes: &[RealRepoKnowledgeScenarioBackendFrontierNodeReceipt],
) -> SubagentFrontierCounts {
    SubagentFrontierCounts {
        judgement_node_count: nodes
            .iter()
            .filter(|node| node.requires_subagent_judgement)
            .count(),
        fanout_node_count: nodes
            .iter()
            .filter(|node| node.subagent_fanout_group_id.is_some())
            .count(),
        fanout_group_count: count_subagent_fanout_groups(nodes),
        max_parallel_width: max_subagent_parallel_width(nodes),
        context_budget_chars: nodes
            .iter()
            .filter_map(|node| node.subagent_context_budget_chars)
            .sum(),
    }
}

pub(super) struct JuliaFrontierCounts {
    pub(super) candidates: usize,
    pub(super) dispatches: usize,
    pub(super) queued: usize,
    pub(super) fallbacks: usize,
    pub(super) rejections: usize,
}

pub(super) fn julia_frontier_counts(
    nodes: &[RealRepoKnowledgeScenarioBackendFrontierNodeReceipt],
) -> JuliaFrontierCounts {
    JuliaFrontierCounts {
        candidates: nodes
            .iter()
            .filter(|node| node.julia_algorithm_id.is_some())
            .count(),
        dispatches: julia_action_count(nodes, "dispatch"),
        queued: julia_action_count(nodes, "queue"),
        fallbacks: julia_action_count(nodes, "fallback"),
        rejections: julia_action_count(nodes, "reject"),
    }
}

fn julia_action_count(
    nodes: &[RealRepoKnowledgeScenarioBackendFrontierNodeReceipt],
    action: &str,
) -> usize {
    nodes
        .iter()
        .filter(|node| node.julia_schedule_action.as_deref() == Some(action))
        .count()
}

pub(super) struct StrategyFlowCounts {
    pub(super) candidate_node_count: usize,
    pub(super) transition_node_count: usize,
    pub(super) frontier_node_count: usize,
    pub(super) context_budget_chars: usize,
    pub(super) cycle_candidate_node_count: usize,
    pub(super) llm_judgement_node_count: usize,
}

pub(super) fn strategy_flow_counts(
    nodes: &[RealRepoKnowledgeScenarioBackendFrontierNodeReceipt],
) -> StrategyFlowCounts {
    StrategyFlowCounts {
        candidate_node_count: nodes
            .iter()
            .filter(|node| node.strategy_flow_candidate_id.is_some())
            .count(),
        transition_node_count: nodes
            .iter()
            .filter(|node| node.strategy_flow_transition_id.is_some())
            .count(),
        frontier_node_count: nodes
            .iter()
            .filter(|node| node.strategy_flow_frontier_rank.is_some())
            .count(),
        context_budget_chars: nodes
            .iter()
            .filter_map(|node| node.strategy_flow_context_budget_chars)
            .sum(),
        cycle_candidate_node_count: nodes
            .iter()
            .filter(|node| node.strategy_flow_loop_candidate)
            .count(),
        llm_judgement_node_count: nodes
            .iter()
            .filter(|node| node.strategy_flow_requires_llm_judgement)
            .count(),
    }
}

fn count_subagent_fanout_groups(
    nodes: &[RealRepoKnowledgeScenarioBackendFrontierNodeReceipt],
) -> usize {
    let mut groups = nodes
        .iter()
        .filter_map(|node| node.subagent_fanout_group_id.as_deref())
        .collect::<Vec<_>>();
    groups.sort_unstable();
    groups.dedup();
    groups.len()
}

fn max_subagent_parallel_width(
    nodes: &[RealRepoKnowledgeScenarioBackendFrontierNodeReceipt],
) -> usize {
    let mut groups = nodes
        .iter()
        .filter_map(|node| node.subagent_fanout_group_id.as_deref())
        .collect::<Vec<_>>();
    groups.sort_unstable();
    groups
        .chunk_by(|left, right| left == right)
        .map(<[_]>::len)
        .max()
        .unwrap_or(0)
}
