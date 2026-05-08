#[cfg(feature = "julia")]
use std::collections::BTreeMap;

use crate::search::real_repo_precision::types::{
    RealRepoKnowledgeScenario, RealRepoKnowledgeScenarioAuthorityReceipt,
    RealRepoKnowledgeScenarioBackendFrontierNodeReceipt,
    RealRepoKnowledgeScenarioBackendFrontierReceipt, RealRepoKnowledgeScenarioNegativeGuardReceipt,
    RealRepoKnowledgeScenarioReasoningTreeReceipt,
    RealRepoKnowledgeScenarioReasoningTreeStepReceipt,
    RealRepoMarkdownKnowledgeSemanticRelationPathReceipt,
};

#[cfg(feature = "julia")]
use xiuxian_polyglot_orchestrator::{
    BenchmarkState, JuliaRuntimeStats, JuliaScheduleAction, JuliaSchedulePlan, JuliaScheduleReason,
    LaneCapability, WarmupState,
};
#[cfg(feature = "julia")]
use xiuxian_wendao_julia::polyglot::{
    JuliaProfileSchedulingFacts, WendaoGraphAlgorithmWorkload, wendaograph_frontier_algorithm_ref,
    wendaograph_frontier_schedule_plan,
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

    let kept_node_count = nodes
        .iter()
        .filter(|node| node.backend_action == "keep")
        .count();
    let pruned_node_count = nodes
        .iter()
        .filter(|node| node.backend_action == "prune")
        .count();
    let expand_node_count = nodes
        .iter()
        .filter(|node| node.backend_action == "expand")
        .count();
    let subagent_judgement_node_count = nodes
        .iter()
        .filter(|node| node.requires_subagent_judgement)
        .count();
    let subagent_fanout_node_count = nodes
        .iter()
        .filter(|node| node.subagent_fanout_group_id.is_some())
        .count();
    let subagent_fanout_group_count = count_subagent_fanout_groups(&nodes);
    let subagent_max_parallel_width = max_subagent_parallel_width(&nodes);
    let subagent_context_budget_chars = nodes
        .iter()
        .filter_map(|node| node.subagent_context_budget_chars)
        .sum();
    let julia_candidate_node_count = nodes
        .iter()
        .filter(|node| node.julia_algorithm_id.is_some())
        .count();
    let julia_dispatch_node_count = nodes
        .iter()
        .filter(|node| node.julia_schedule_action.as_deref() == Some("dispatch"))
        .count();
    let julia_queue_node_count = nodes
        .iter()
        .filter(|node| node.julia_schedule_action.as_deref() == Some("queue"))
        .count();
    let julia_fallback_node_count = nodes
        .iter()
        .filter(|node| node.julia_schedule_action.as_deref() == Some("fallback"))
        .count();
    let julia_reject_node_count = nodes
        .iter()
        .filter(|node| node.julia_schedule_action.as_deref() == Some("reject"))
        .count();
    let strategy_flow_candidate_node_count = nodes
        .iter()
        .filter(|node| node.strategy_flow_candidate_id.is_some())
        .count();
    let strategy_flow_transition_node_count = nodes
        .iter()
        .filter(|node| node.strategy_flow_transition_id.is_some())
        .count();
    let strategy_flow_frontier_node_count = nodes
        .iter()
        .filter(|node| node.strategy_flow_frontier_rank.is_some())
        .count();
    let strategy_flow_context_budget_chars = nodes
        .iter()
        .filter_map(|node| node.strategy_flow_context_budget_chars)
        .sum();
    let strategy_flow_cycle_candidate_node_count = nodes
        .iter()
        .filter(|node| node.strategy_flow_loop_candidate)
        .count();
    let strategy_flow_llm_judgement_node_count = nodes
        .iter()
        .filter(|node| node.strategy_flow_requires_llm_judgement)
        .count();
    let strategy_flow_loop_budget =
        strategy_flow_loop_budget(scenario, strategy_flow_cycle_candidate_node_count);
    let strategy_flow_refinement_topology = strategy_flow_refinement_topology(
        strategy_flow_cycle_candidate_node_count,
        strategy_flow_llm_judgement_node_count,
    );

    RealRepoKnowledgeScenarioBackendFrontierReceipt {
        strategy: "rust_controlled_backend_frontier_v1".to_string(),
        control_plane_owner: "rust".to_string(),
        graph_backend: "rust-baseline".to_string(),
        graph_backend_live: false,
        julia_schedule_basis: julia_schedule_basis().to_string(),
        node_count: nodes.len(),
        kept_node_count,
        pruned_node_count,
        expand_node_count,
        subagent_judgement_node_count,
        subagent_fanout_group_count,
        subagent_fanout_node_count,
        subagent_max_parallel_width,
        subagent_context_budget_chars,
        julia_candidate_node_count,
        julia_dispatch_node_count,
        julia_queue_node_count,
        julia_fallback_node_count,
        julia_reject_node_count,
        strategy_flow_projection_basis: "rust_receipt_projection_v1".to_string(),
        strategy_flow_candidate_node_count,
        strategy_flow_transition_node_count,
        strategy_flow_frontier_node_count,
        strategy_flow_context_budget_chars,
        strategy_flow_intent_complexity_class: strategy_flow_intent_complexity_class(scenario)
            .to_string(),
        strategy_flow_initial_topology: "acyclic_evidence_dag".to_string(),
        strategy_flow_refinement_topology: strategy_flow_refinement_topology.to_string(),
        strategy_flow_max_planned_depth: strategy_flow_max_planned_depth(scenario),
        strategy_flow_loop_budget,
        strategy_flow_cycle_candidate_node_count,
        strategy_flow_llm_judgement_node_count,
        selected_beam_width: kept_node_count + expand_node_count,
        nodes,
    }
}

fn node_from_reasoning_step(
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

fn authority_node(
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

fn negative_guard_node(
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

fn attach_search_strategy_flow_projection(
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
    base.saturating_sub((node.context_cost as u32).min(1_500))
}

fn strategy_flow_intent_complexity_class(scenario: &RealRepoKnowledgeScenario) -> &'static str {
    let has_graph_hops = !scenario.required_relation_paths.is_empty()
        || !scenario.required_semantic_object_ids.is_empty();
    let has_guard = scenario.authority.is_some() || !scenario.forbidden_paths.is_empty();
    if has_graph_hops && has_guard {
        return "guarded_multi_hop";
    }
    if matches!(
        scenario.kind,
        crate::search::real_repo_precision::types::RealRepoKnowledgeScenarioKind::AgentTask
    ) {
        return "agentic";
    }
    if has_graph_hops {
        return "multi_hop_graph";
    }
    if matches!(
        scenario.kind,
        crate::search::real_repo_precision::types::RealRepoKnowledgeScenarioKind::NaturalLanguageIntent
            | crate::search::real_repo_precision::types::RealRepoKnowledgeScenarioKind::AmbiguousAlias
    ) {
        return "natural_language";
    }
    "known_item"
}

fn strategy_flow_max_planned_depth(scenario: &RealRepoKnowledgeScenario) -> usize {
    if !scenario.required_relation_paths.is_empty() || !scenario.required_semantic_object_ids.is_empty()
    {
        3
    } else if matches!(
        scenario.kind,
        crate::search::real_repo_precision::types::RealRepoKnowledgeScenarioKind::NaturalLanguageIntent
            | crate::search::real_repo_precision::types::RealRepoKnowledgeScenarioKind::AmbiguousAlias
            | crate::search::real_repo_precision::types::RealRepoKnowledgeScenarioKind::AgentTask
    ) {
        2
    } else {
        1
    }
}

fn strategy_flow_loop_budget(
    scenario: &RealRepoKnowledgeScenario,
    cycle_candidate_node_count: usize,
) -> usize {
    if cycle_candidate_node_count == 0 {
        return 0;
    }
    if scenario.required_relation_paths.len() > 1
        || matches!(
            scenario.kind,
            crate::search::real_repo_precision::types::RealRepoKnowledgeScenarioKind::AgentTask
                | crate::search::real_repo_precision::types::RealRepoKnowledgeScenarioKind::AmbiguousAlias
        )
    {
        return 2;
    }
    1
}

fn strategy_flow_refinement_topology(
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

#[cfg(feature = "julia")]
fn attach_julia_schedule_projection(
    nodes: &mut [RealRepoKnowledgeScenarioBackendFrontierNodeReceipt],
) {
    let mut rows_by_algorithm = BTreeMap::<String, u32>::new();
    let mut bytes_by_algorithm = BTreeMap::<String, u64>::new();
    let frontier_node_count = saturating_usize_to_u32(nodes.len());
    let frontier_edge_count = estimated_frontier_edge_count(nodes);

    for node in nodes.iter() {
        if node.backend_action == "prune" {
            continue;
        }
        if let Some(algorithm) = wendaograph_frontier_algorithm_ref(node.evidence_kind.as_str()) {
            let algorithm_id = algorithm.algorithm_id.to_string();
            *rows_by_algorithm.entry(algorithm_id.clone()).or_insert(0) += 1;
            *bytes_by_algorithm.entry(algorithm_id).or_insert(0) += estimated_node_byte_size(node);
        }
    }

    for node in nodes.iter_mut() {
        if node.backend_action == "prune" {
            continue;
        }
        let Some(algorithm) = wendaograph_frontier_algorithm_ref(node.evidence_kind.as_str())
        else {
            continue;
        };
        let rows = rows_by_algorithm
            .get(algorithm.algorithm_id)
            .copied()
            .unwrap_or(1);
        let byte_size = bytes_by_algorithm
            .get(algorithm.algorithm_id)
            .copied()
            .unwrap_or_else(|| estimated_node_byte_size(node));
        let workload = WendaoGraphAlgorithmWorkload::new()
            .with_rows(rows)
            .with_graph_size(frontier_node_count, frontier_edge_count)
            .with_feature_columns(8)
            .with_byte_size(byte_size);
        let facts = static_warm_profile_schedule_facts();
        let Some(plan) =
            wendaograph_frontier_schedule_plan(node.evidence_kind.as_str(), workload, facts)
        else {
            continue;
        };
        apply_julia_schedule_projection(node, algorithm.algorithm_id, plan);
    }
}

#[cfg(not(feature = "julia"))]
fn attach_julia_schedule_projection(
    _nodes: &mut [RealRepoKnowledgeScenarioBackendFrontierNodeReceipt],
) {
}

#[cfg(feature = "julia")]
fn apply_julia_schedule_projection(
    node: &mut RealRepoKnowledgeScenarioBackendFrontierNodeReceipt,
    algorithm_id: &str,
    plan: JuliaSchedulePlan,
) {
    node.julia_algorithm_id = Some(algorithm_id.to_string());
    node.julia_profile_id = Some(plan.profile_id);
    node.julia_capability = Some(lane_capability_id(plan.capability).to_string());
    node.julia_schedule_action = Some(schedule_action_id(plan.action).to_string());
    node.julia_schedule_reason = Some(schedule_reason_id(plan.reason).to_string());
    node.julia_schedule_confidence_score = Some(plan.confidence_score);
    node.julia_selected_batch_size = Some(plan.selected_batch_size);
}

#[cfg(feature = "julia")]
fn static_warm_profile_schedule_facts() -> JuliaProfileSchedulingFacts {
    JuliaProfileSchedulingFacts::new(
        JuliaRuntimeStats::new()
            .with_warmup(WarmupState::Ready)
            .with_benchmark(BenchmarkState::WithinThreshold)
            .with_latency_ms(Some(3), Some(8)),
    )
    .with_max_in_flight(Some(4))
    .with_fallback_available(true)
    .with_target_latency_ms(Some(250))
}

#[cfg(feature = "julia")]
fn lane_capability_id(capability: LaneCapability) -> &'static str {
    match capability {
        LaneCapability::DocumentExtraction => "document_extraction",
        LaneCapability::OcrShardExtraction => "ocr_shard_extraction",
        LaneCapability::GraphEvidenceCompute => "graph_evidence_compute",
        LaneCapability::GraphSearchCompute => "graph_search_compute",
        LaneCapability::ScientificCompute => "scientific_compute",
        LaneCapability::MemoryProfileCompute => "memory_profile_compute",
    }
}

#[cfg(feature = "julia")]
fn schedule_action_id(action: JuliaScheduleAction) -> &'static str {
    match action {
        JuliaScheduleAction::Dispatch => "dispatch",
        JuliaScheduleAction::Queue => "queue",
        JuliaScheduleAction::Fallback => "fallback",
        JuliaScheduleAction::Reject => "reject",
    }
}

#[cfg(feature = "julia")]
fn schedule_reason_id(reason: JuliaScheduleReason) -> &'static str {
    match reason {
        JuliaScheduleReason::JuliaAdvantage => "julia_advantage",
        JuliaScheduleReason::JuliaWarming => "julia_warming",
        JuliaScheduleReason::JuliaAtCapacity => "julia_at_capacity",
        JuliaScheduleReason::ContractInvalid => "contract_invalid",
        JuliaScheduleReason::BenchmarkFailed => "benchmark_failed",
        JuliaScheduleReason::RuntimeUnstable => "runtime_unstable",
        JuliaScheduleReason::QueuePressure => "queue_pressure",
        JuliaScheduleReason::NoCapacity => "no_capacity",
        JuliaScheduleReason::DeadlineTooTight => "deadline_too_tight",
        JuliaScheduleReason::CostExceedsBenefit => "cost_exceeds_benefit",
    }
}

#[cfg(feature = "julia")]
fn estimated_node_byte_size(node: &RealRepoKnowledgeScenarioBackendFrontierNodeReceipt) -> u64 {
    (node.context_cost.max(1) as u64).saturating_mul(64)
}

#[cfg(feature = "julia")]
fn estimated_frontier_edge_count(
    nodes: &[RealRepoKnowledgeScenarioBackendFrontierNodeReceipt],
) -> u32 {
    let parent_edge_count = nodes
        .iter()
        .filter(|node| node.parent_node_id.is_some())
        .count();
    saturating_usize_to_u32(parent_edge_count + nodes.len().saturating_sub(1))
}

#[cfg(feature = "julia")]
fn saturating_usize_to_u32(value: usize) -> u32 {
    value.try_into().unwrap_or(u32::MAX)
}

#[cfg(feature = "julia")]
fn julia_schedule_basis() -> &'static str {
    "static_warm_profile_projection_v1"
}

#[cfg(not(feature = "julia"))]
fn julia_schedule_basis() -> &'static str {
    "disabled"
}

fn evidence_kind_for_step(
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

fn backend_action_for_step(
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
    let mut max_width = 0;
    let mut index = 0;
    while index < groups.len() {
        let group = groups[index];
        let mut width = 1;
        index += 1;
        while index < groups.len() && groups[index] == group {
            width += 1;
            index += 1;
        }
        max_width = max_width.max(width);
    }
    max_width
}

fn subagent_priority_score_bps(
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
    let context_penalty = (step_context_cost(step) as u32).min(1_500);
    ((graph_score + authority_score + coverage_score) / 3).saturating_sub(context_penalty)
}

fn subagent_context_budget_chars(
    step: &RealRepoKnowledgeScenarioReasoningTreeStepReceipt,
) -> usize {
    let base = 512 + step_context_cost(step).saturating_mul(2);
    base.clamp(640, 1_600)
}

fn graph_score_for_step(step: &RealRepoKnowledgeScenarioReasoningTreeStepReceipt) -> u32 {
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
    let rank_penalty = step.zero_based_rank.unwrap_or_default().min(10) as u32 * 250;
    base.saturating_sub(rank_penalty)
}

fn authority_score_for_step(
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

fn parent_node_id(
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

fn node_id(scenario: &RealRepoKnowledgeScenario, step_index: usize) -> String {
    format!("frontier:{}:step:{step_index}", scenario.id)
}

fn step_context_cost(step: &RealRepoKnowledgeScenarioReasoningTreeStepReceipt) -> usize {
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
