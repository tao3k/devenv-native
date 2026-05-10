use super::{
    JuliaProfileSchedulingFacts, MemoryJuliaComputeReadinessInput,
    WENDAO_GRAPH_GNN_REASONING_HOST_ENTRYPOINT, WENDAO_GRAPH_GNN_REASONING_PROFILE_ID,
    WENDAO_GRAPH_GNN_REASONING_SCHEMA_VERSION, WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_HOST_ENTRYPOINT,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID, WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID,
    WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID, WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID,
    WendaoGraphAlgorithmId, WendaoGraphAlgorithmWorkload, WendaoGraphProfileId,
    WendaoGraphReadinessInput, WendaoSearchGraphStructuralReadinessInput,
    WendaoSearchLegacyRerankReadinessInput, julia_graph_compute_profile_refs,
    julia_graph_compute_snapshot, memory_julia_compute_config_readiness,
    memory_julia_compute_manifest_row_ref, memory_julia_compute_profile_ref,
    memory_julia_compute_profile_refs, memory_julia_compute_schedule_plan,
    memory_julia_compute_snapshot, wendao_graph_gnn_accelerator_diagnostics_from_host_probe,
    wendao_graph_gnn_readiness_evidence_from_host_probe, wendao_graph_gnn_reasoning_profile_ref,
    wendao_graph_gnn_reasoning_schedule_plan, wendao_graph_gnn_runtime_stats_from_host_probe,
    wendao_graph_link_evidence_profile_ref,
    wendao_graph_link_evidence_readiness_evidence_from_full_structural_host_probe,
    wendao_graph_link_evidence_runtime_stats_from_full_structural_host_probe,
    wendao_graph_link_evidence_schedule_plan, wendao_graph_page_index_reasoning_profile_ref,
    wendao_graph_page_index_reasoning_readiness_evidence_from_host_probe,
    wendao_graph_page_index_reasoning_runtime_stats_from_host_probe,
    wendao_graph_page_index_reasoning_runtime_stats_from_planner_action_host_probe,
    wendao_graph_page_index_reasoning_schedule_plan, wendaograph_algorithm_refs,
    wendaograph_frontier_algorithm_ref, wendaograph_frontier_schedule_plan,
    wendaograph_frontier_task_shape, wendaograph_gnn_algorithm_refs,
    wendaograph_link_graph_algorithm_refs, wendaograph_page_index_algorithm_refs,
    wendaograph_relationship_search_algorithm_refs,
    wendaograph_relationship_search_evidence_from_full_structural_host_probe,
    wendaograph_search_strategy_flow_algorithm_refs, wendaosearch_graph_structural_profile_ref,
    wendaosearch_graph_structural_schedule_plan, wendaosearch_legacy_rerank_profile_ref,
    wendaosearch_legacy_rerank_schedule_plan, with_julia_thread_pinning_diagnostics,
};
use crate::compatibility::link_graph::{
    DEFAULT_JULIA_RERANK_FLIGHT_ROUTE, LinkGraphJuliaRerankRuntimeConfig,
};
use crate::integration_support::{
    WendaoGraphGnnBackendLoadDiagnostics, WendaoGraphGnnHostProbeReport,
    WendaoGraphLinkGraphFullStructuralHostProbeReport, WendaoGraphLinkGraphHostProbeReport,
    WendaoGraphPageIndexHostProbeReport, WendaoGraphPageIndexPlannerActionHostProbeReport,
};
use crate::memory::{
    MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID, MemoryJuliaComputeManifestRow,
    MemoryJuliaComputeProfile,
};
use crate::{
    GRAPH_STRUCTURAL_FILTER_ROUTE, GRAPH_STRUCTURAL_RERANK_ROUTE, GraphStructuralRouteKind,
    JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION, WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION,
    WENDAO_GRAPH_LINK_EVIDENCE_ROUTE,
};
use xiuxian_polyglot_orchestrator::{
    AdmissionDecision, BenchmarkState, ContractOwner, JuliaComputeTaskShape, JuliaRuntimeStats,
    JuliaScheduleAction, JuliaScheduleReason, JuliaTaskComplexityClass,
    JuliaThreadPinningDiagnostics, JuliaThreadPinningState, JuliaThreadTopology, LaneCapability,
    PolyglotLane, ReadinessState, RejectionReason, SnapshotInvariantError, WarmupState,
};
use xiuxian_wendao_runtime::config::{
    MemoryJuliaComputeFallbackMode, MemoryJuliaComputeRuntimeConfig,
};

fn required<T>(value: Option<T>, label: &str) -> T {
    value.unwrap_or_else(|| panic!("missing required {label}"))
}

fn wendao_graph_link_evidence_readiness_evidence(
    warmup: WarmupState,
    benchmark: BenchmarkState,
    max_in_flight: Option<u32>,
    active_in_flight: u32,
    queue_depth: u32,
) -> xiuxian_polyglot_orchestrator::JuliaReadinessEvidence {
    super::wendao_graph_link_evidence_readiness_evidence(WendaoGraphReadinessInput {
        warmup,
        benchmark,
        max_in_flight,
        active_in_flight,
        queue_depth,
    })
}

fn wendao_graph_page_index_reasoning_readiness_evidence(
    warmup: WarmupState,
    benchmark: BenchmarkState,
    max_in_flight: Option<u32>,
    active_in_flight: u32,
    queue_depth: u32,
) -> xiuxian_polyglot_orchestrator::JuliaReadinessEvidence {
    super::wendao_graph_page_index_reasoning_readiness_evidence(WendaoGraphReadinessInput {
        warmup,
        benchmark,
        max_in_flight,
        active_in_flight,
        queue_depth,
    })
}

fn wendao_graph_gnn_reasoning_readiness_evidence(
    warmup: WarmupState,
    benchmark: BenchmarkState,
    max_in_flight: Option<u32>,
    active_in_flight: u32,
    queue_depth: u32,
) -> xiuxian_polyglot_orchestrator::JuliaReadinessEvidence {
    super::wendao_graph_gnn_reasoning_readiness_evidence(WendaoGraphReadinessInput {
        warmup,
        benchmark,
        max_in_flight,
        active_in_flight,
        queue_depth,
    })
}

fn wendaosearch_graph_structural_readiness_evidence(
    route_kind: GraphStructuralRouteKind,
    warmup: WarmupState,
    benchmark: BenchmarkState,
    max_in_flight: Option<u32>,
    active_in_flight: u32,
    queue_depth: u32,
) -> xiuxian_polyglot_orchestrator::JuliaReadinessEvidence {
    super::wendaosearch_graph_structural_readiness_evidence(
        WendaoSearchGraphStructuralReadinessInput {
            route_kind,
            warmup,
            benchmark,
            max_in_flight,
            active_in_flight,
            queue_depth,
        },
    )
}

fn wendaosearch_legacy_rerank_readiness_evidence(
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
    warmup: WarmupState,
    benchmark: BenchmarkState,
    active_in_flight: u32,
    queue_depth: u32,
) -> xiuxian_polyglot_orchestrator::JuliaReadinessEvidence {
    super::wendaosearch_legacy_rerank_readiness_evidence(WendaoSearchLegacyRerankReadinessInput {
        runtime,
        warmup,
        benchmark,
        active_in_flight,
        queue_depth,
    })
}

fn memory_julia_compute_readiness_evidence(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    profile: MemoryJuliaComputeProfile,
    warmup: WarmupState,
    benchmark: BenchmarkState,
    active_in_flight: u32,
    queue_depth: u32,
) -> xiuxian_polyglot_orchestrator::JuliaReadinessEvidence {
    super::memory_julia_compute_readiness_evidence(MemoryJuliaComputeReadinessInput {
        runtime,
        profile,
        warmup,
        benchmark,
        active_in_flight,
        queue_depth,
    })
}

fn memory_julia_compute_readiness_snapshot(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    profile: MemoryJuliaComputeProfile,
    warmup: WarmupState,
    benchmark: BenchmarkState,
    active_in_flight: u32,
    queue_depth: u32,
) -> Result<xiuxian_polyglot_orchestrator::PolyglotControlSnapshot, SnapshotInvariantError> {
    super::memory_julia_compute_readiness_snapshot(MemoryJuliaComputeReadinessInput {
        runtime,
        profile,
        warmup,
        benchmark,
        active_in_flight,
        queue_depth,
    })
}

fn wendaograph_algorithm_ref(algorithm_id: &'static str) -> Option<super::WendaoGraphAlgorithmRef> {
    super::wendaograph_algorithm_ref(WendaoGraphAlgorithmId(algorithm_id))
}

fn wendaograph_algorithm_task_shape(
    algorithm_id: &'static str,
    workload: WendaoGraphAlgorithmWorkload,
) -> Option<JuliaComputeTaskShape> {
    super::wendaograph_algorithm_task_shape(WendaoGraphAlgorithmId(algorithm_id), workload)
}

fn wendaograph_algorithm_refs_for_profile(
    profile_id: &'static str,
) -> Vec<super::WendaoGraphAlgorithmRef> {
    super::wendaograph_algorithm_refs_for_profile(WendaoGraphProfileId(profile_id))
}

fn wendaograph_algorithm_schedule_plan(
    algorithm_id: &'static str,
    workload: WendaoGraphAlgorithmWorkload,
    facts: JuliaProfileSchedulingFacts,
) -> Option<xiuxian_polyglot_orchestrator::JuliaSchedulePlan> {
    super::wendaograph_algorithm_schedule_plan(
        WendaoGraphAlgorithmId(algorithm_id),
        workload,
        facts,
    )
}

fn wendaograph_relationship_search_evidence_for_algorithm_from_full_structural_host_probe(
    algorithm_id: &'static str,
    report: &WendaoGraphLinkGraphFullStructuralHostProbeReport,
    workload: WendaoGraphAlgorithmWorkload,
    facts: JuliaProfileSchedulingFacts,
) -> Option<super::WendaoGraphRelationshipSearchEvidence> {
    super::wendaograph_relationship_search_evidence_for_algorithm_from_full_structural_host_probe(
        WendaoGraphAlgorithmId(algorithm_id),
        report,
        workload,
        facts,
    )
}

mod graph_catalog;
mod graph_readiness;
mod graph_scheduling;
mod memory;
mod profile_refs;
mod schedule_cases;

fn scheduling_facts(warmup: WarmupState, benchmark: BenchmarkState) -> JuliaProfileSchedulingFacts {
    JuliaProfileSchedulingFacts::new(
        JuliaRuntimeStats::new()
            .with_warmup(warmup)
            .with_benchmark(benchmark)
            .with_latency_ms(Some(30), Some(90)),
    )
}

fn heavy_graph_shape() -> JuliaComputeTaskShape {
    JuliaComputeTaskShape::new()
        .with_rows(24)
        .with_graph_size(1_500, 12_000)
        .with_feature_columns(18)
        .with_byte_size(2 * 1024 * 1024)
        .with_batchability_key("graph:v1")
        .with_complexity(JuliaTaskComplexityClass::Heavy)
}

fn memory_shape() -> JuliaComputeTaskShape {
    JuliaComputeTaskShape::new()
        .with_rows(1)
        .with_feature_columns(6)
        .with_byte_size(64 * 1024)
        .with_complexity(JuliaTaskComplexityClass::Simple)
}

fn graph_algorithm_workload() -> WendaoGraphAlgorithmWorkload {
    WendaoGraphAlgorithmWorkload::new()
        .with_rows(24)
        .with_graph_size(1_500, 12_000)
        .with_feature_columns(18)
        .with_byte_size(2 * 1024 * 1024)
}

fn relationship_search_evidence_row<'a>(
    evidence: &'a [super::WendaoGraphRelationshipSearchEvidence],
    algorithm_id: &str,
) -> &'a super::WendaoGraphRelationshipSearchEvidence {
    evidence
        .iter()
        .find(|row| row.algorithm.algorithm_id == algorithm_id)
        .unwrap_or_else(|| panic!("missing relationship search evidence for {algorithm_id}"))
}

fn gnn_host_probe_report() -> WendaoGraphGnnHostProbeReport {
    WendaoGraphGnnHostProbeReport {
        sample_count: 2,
        first_ms: 26_186.319,
        warm_min_ms: 18.735,
        warm_median_ms: 18.735,
        warm_p95_ms: 388.719,
        warm_max_ms: 388.719,
        node_count: 4,
        edge_count: 4,
        feature_rows: 7,
        feature_cols: 4,
        score_count: 4,
        frontier_rows: 3,
        backend_load: WendaoGraphGnnBackendLoadDiagnostics {
            metal_loaded: true,
            cuda_loaded: false,
            amdgpu_loaded: false,
        },
        metal_functional: true,
        metal_score_count: 4,
    }
}

fn link_graph_full_structural_host_probe_report()
-> WendaoGraphLinkGraphFullStructuralHostProbeReport {
    WendaoGraphLinkGraphFullStructuralHostProbeReport {
        base: WendaoGraphLinkGraphHostProbeReport {
            mode: "semantic-neighbors".into(),
            node_count: 4,
            edge_count: 2,
            semantic_neighbor_count: 1,
            sample_count: 3,
            first_ms: 9_485.249,
            warm_min_ms: 0.555,
            warm_median_ms: 0.555,
            warm_p95_ms: 0.742,
            warm_max_ms: 0.742,
            graph_metric_rows: 4,
            topology_candidate_rows: 1,
            semantic_overlay_rows: 2,
            diffusion_rows: 4,
            frontier_rows: 3,
        },
        component_rows: 4,
        topology_profile_rows: 4,
        topology_bottleneck_rows: 4,
        topology_community_rows: 4,
        topology_cover_rows: 4,
        topology_core_rows: 4,
        topology_boundary_rows: 4,
        topology_transition_rows: 2,
        topology_gateway_rows: 4,
        topology_community_summary_rows: 2,
        topology_community_link_rows: 0,
        topology_community_frontier_rows: 1,
    }
}

fn page_index_host_probe_report() -> WendaoGraphPageIndexHostProbeReport {
    WendaoGraphPageIndexHostProbeReport {
        sample_count: 3,
        first_ms: 1_776.453,
        warm_min_ms: 0.022,
        warm_median_ms: 0.022,
        warm_p95_ms: 0.129,
        warm_max_ms: 0.129,
        frontier_rows: 3,
        trace_rows: 3,
    }
}

fn page_index_planner_action_host_probe_report() -> WendaoGraphPageIndexPlannerActionHostProbeReport
{
    WendaoGraphPageIndexPlannerActionHostProbeReport {
        base: page_index_host_probe_report(),
        planner_action_rows: 3,
        planner_expand_actions: 1,
        planner_compare_actions: 0,
        planner_jump_actions: 1,
        planner_stop_actions: 1,
    }
}
