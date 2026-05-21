//! Attach Julia scheduling evidence to graph-oriented frontier nodes.

use crate::search::real_repo_precision::RealRepoKnowledgeScenarioBackendFrontierNodeReceipt;

#[cfg(feature = "julia")]
use std::collections::BTreeMap;

#[cfg(feature = "julia")]
use crate::search::real_repo_precision::frontier::score::saturating_usize_to_u32;
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

#[cfg(feature = "julia")]
pub(super) fn attach_julia_schedule_projection(
    nodes: &mut [RealRepoKnowledgeScenarioBackendFrontierNodeReceipt],
) {
    let context = JuliaFrontierScheduleContext::from_nodes(nodes);
    for node in nodes {
        attach_julia_schedule_projection_to_node(node, &context);
    }
}

#[cfg(not(feature = "julia"))]
pub(super) fn attach_julia_schedule_projection(
    _nodes: &mut [RealRepoKnowledgeScenarioBackendFrontierNodeReceipt],
) {
}

#[cfg(feature = "julia")]
pub(super) fn julia_schedule_basis() -> &'static str {
    "static_warm_profile_projection_v1"
}

#[cfg(not(feature = "julia"))]
pub(super) fn julia_schedule_basis() -> &'static str {
    "disabled"
}

#[cfg(feature = "julia")]
#[derive(Debug)]
struct JuliaFrontierScheduleContext {
    rows_by_algorithm: BTreeMap<String, u32>,
    bytes_by_algorithm: BTreeMap<String, u64>,
    frontier_node_count: u32,
    frontier_edge_count: u32,
}

#[cfg(feature = "julia")]
impl JuliaFrontierScheduleContext {
    fn from_nodes(nodes: &[RealRepoKnowledgeScenarioBackendFrontierNodeReceipt]) -> Self {
        let (rows_by_algorithm, bytes_by_algorithm) = collect_algorithm_workload_totals(nodes);
        Self {
            rows_by_algorithm,
            bytes_by_algorithm,
            frontier_node_count: saturating_usize_to_u32(nodes.len()),
            frontier_edge_count: estimated_frontier_edge_count(nodes),
        }
    }

    fn workload_for(
        &self,
        algorithm_id: &str,
        node: &RealRepoKnowledgeScenarioBackendFrontierNodeReceipt,
    ) -> WendaoGraphAlgorithmWorkload {
        WendaoGraphAlgorithmWorkload::new()
            .with_rows(self.rows_for(algorithm_id))
            .with_graph_size(self.frontier_node_count, self.frontier_edge_count)
            .with_feature_columns(8)
            .with_byte_size(self.byte_size_for(algorithm_id, node))
    }

    fn rows_for(&self, algorithm_id: &str) -> u32 {
        self.rows_by_algorithm
            .get(algorithm_id)
            .copied()
            .unwrap_or(1)
    }

    fn byte_size_for(
        &self,
        algorithm_id: &str,
        node: &RealRepoKnowledgeScenarioBackendFrontierNodeReceipt,
    ) -> u64 {
        self.bytes_by_algorithm
            .get(algorithm_id)
            .copied()
            .unwrap_or_else(|| estimated_node_byte_size(node))
    }
}

#[cfg(feature = "julia")]
fn collect_algorithm_workload_totals(
    nodes: &[RealRepoKnowledgeScenarioBackendFrontierNodeReceipt],
) -> (BTreeMap<String, u32>, BTreeMap<String, u64>) {
    let mut rows_by_algorithm = BTreeMap::<String, u32>::new();
    let mut bytes_by_algorithm = BTreeMap::<String, u64>::new();

    nodes
        .iter()
        .filter(|node| node.backend_action != "prune")
        .filter_map(|node| {
            let algorithm = wendaograph_frontier_algorithm_ref(node.evidence_kind.as_str())?;
            Some((
                algorithm.algorithm_id.to_string(),
                estimated_node_byte_size(node),
            ))
        })
        .for_each(|(algorithm_id, byte_size)| {
            *rows_by_algorithm.entry(algorithm_id.clone()).or_insert(0) += 1;
            *bytes_by_algorithm.entry(algorithm_id).or_insert(0) += byte_size;
        });

    (rows_by_algorithm, bytes_by_algorithm)
}

#[cfg(feature = "julia")]
fn attach_julia_schedule_projection_to_node(
    node: &mut RealRepoKnowledgeScenarioBackendFrontierNodeReceipt,
    context: &JuliaFrontierScheduleContext,
) {
    if node.backend_action == "prune" {
        return;
    }

    let Some(algorithm) = wendaograph_frontier_algorithm_ref(node.evidence_kind.as_str()) else {
        return;
    };
    let workload = context.workload_for(algorithm.algorithm_id, node);
    let facts = static_warm_profile_schedule_facts();

    if let Some(plan) =
        wendaograph_frontier_schedule_plan(node.evidence_kind.as_str(), workload, facts)
    {
        apply_julia_schedule_projection(node, algorithm.algorithm_id, plan);
    }
}

#[cfg(feature = "julia")]
fn apply_julia_schedule_projection(
    node: &mut RealRepoKnowledgeScenarioBackendFrontierNodeReceipt,
    algorithm_id: &str,
    plan: JuliaSchedulePlan,
) {
    node.julia_algorithm_id = Some(algorithm_id.to_string());
    node.julia_profile_id = Some(plan.profile_id.into_string());
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
