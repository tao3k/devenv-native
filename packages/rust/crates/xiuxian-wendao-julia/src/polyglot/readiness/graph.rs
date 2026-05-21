//! `WendaoGraph` readiness and scheduling projections.

use xiuxian_polyglot_orchestrator::{
    BenchmarkState, JuliaAcceleratorDiagnostics, JuliaComputeTaskShape, JuliaReadinessEvidence,
    JuliaRuntimeStats, JuliaSchedulePlan, LaneCapability, WarmupState,
};

use crate::WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION;
use crate::integration_support::{
    WendaoGraphGnnHostProbeReport, WendaoGraphLinkGraphFullStructuralHostProbeReport,
    WendaoGraphLinkGraphHostProbeReport, WendaoGraphPageIndexHostProbeReport,
    WendaoGraphPageIndexPlannerActionHostProbeReport,
};
use crate::polyglot::state::{
    JuliaProfileSchedulingFacts, WENDAO_GRAPH_GNN_REASONING_PROFILE_ID,
    WENDAO_GRAPH_GNN_REASONING_SCHEMA_VERSION, WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID, WendaoGraphRelationshipSearchEvidence,
};
use crate::polyglot::wendaograph_algorithms::{
    WendaoGraphAlgorithmId, WendaoGraphAlgorithmWorkload, wendaograph_algorithm_ref,
    wendaograph_frontier_algorithm_ref, wendaograph_relationship_search_algorithm_refs,
};

use super::evidence_support::{
    JuliaReadinessWindow, JuliaStaticContractReadinessProfile, julia_schedule_plan_from_readiness,
    julia_static_contract_readiness_evidence, latency_ms_as_u32, saturating_usize_to_u32,
};

/// Readiness facts shared by the `WendaoGraph.jl` graph profiles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WendaoGraphReadinessInput {
    /// Julia warmup state observed by the owner.
    pub warmup: WarmupState,
    /// Benchmark state observed by the owner.
    pub benchmark: BenchmarkState,
    /// Optional maximum concurrent request budget.
    pub max_in_flight: Option<u32>,
    /// Active in-flight request count.
    pub active_in_flight: u32,
    /// Queued request count.
    pub queue_depth: u32,
}

impl WendaoGraphReadinessInput {
    fn window(self) -> JuliaReadinessWindow {
        JuliaReadinessWindow {
            max_in_flight: self.max_in_flight,
            active_in_flight: self.active_in_flight,
            queue_depth: self.queue_depth,
        }
    }
}

/// Returns readiness evidence for the `WendaoGraph.jl` link-evidence profile.
#[must_use]
pub fn wendao_graph_link_evidence_readiness_evidence(
    input: WendaoGraphReadinessInput,
) -> JuliaReadinessEvidence {
    julia_static_contract_readiness_evidence(
        JuliaStaticContractReadinessProfile {
            capability: LaneCapability::GraphEvidenceCompute,
            profile_id: WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
            schema_version: WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION,
        },
        input.warmup,
        input.benchmark,
        input.window(),
    )
}

/// Returns a schedule plan for the `WendaoGraph.jl` link-evidence profile.
#[must_use]
pub fn wendao_graph_link_evidence_schedule_plan(
    shape: JuliaComputeTaskShape,
    facts: JuliaProfileSchedulingFacts,
) -> JuliaSchedulePlan {
    let readiness = wendao_graph_link_evidence_readiness_evidence(WendaoGraphReadinessInput {
        warmup: facts.runtime_stats.warmup,
        benchmark: facts.runtime_stats.benchmark,
        max_in_flight: facts.max_in_flight,
        active_in_flight: facts.runtime_stats.active_in_flight,
        queue_depth: facts.runtime_stats.queue_depth,
    })
    .with_fallback_available(facts.fallback_available);
    julia_schedule_plan_from_readiness(readiness, shape, facts)
}

/// Returns runtime stats derived from a `WendaoGraph.jl` `LinkGraph` host probe
/// report.
#[must_use]
pub fn wendao_graph_link_evidence_runtime_stats_from_host_probe(
    report: &WendaoGraphLinkGraphHostProbeReport,
) -> JuliaRuntimeStats {
    JuliaRuntimeStats::new()
        .with_warmup(WarmupState::Ready)
        .with_benchmark(BenchmarkState::NotRequired)
        .with_latency_ms(
            Some(latency_ms_as_u32(report.warm_median_ms)),
            Some(latency_ms_as_u32(report.warm_p95_ms)),
        )
}

/// Returns runtime stats derived from a full-structural `WendaoGraph.jl`
/// `LinkGraph` host probe report.
#[must_use]
pub fn wendao_graph_link_evidence_runtime_stats_from_full_structural_host_probe(
    report: &WendaoGraphLinkGraphFullStructuralHostProbeReport,
) -> JuliaRuntimeStats {
    wendao_graph_link_evidence_runtime_stats_from_host_probe(&report.base)
}

/// Returns readiness evidence derived from a full-structural `WendaoGraph.jl`
/// `LinkGraph` host probe report.
#[must_use]
pub fn wendao_graph_link_evidence_readiness_evidence_from_full_structural_host_probe(
    report: &WendaoGraphLinkGraphFullStructuralHostProbeReport,
    max_in_flight: Option<u32>,
    active_in_flight: u32,
    queue_depth: u32,
) -> JuliaReadinessEvidence {
    let benchmark = if report.base.sample_count > 0 {
        BenchmarkState::NotRequired
    } else {
        BenchmarkState::Failed
    };
    wendao_graph_link_evidence_readiness_evidence(WendaoGraphReadinessInput {
        warmup: WarmupState::Ready,
        benchmark,
        max_in_flight,
        active_in_flight,
        queue_depth,
    })
}

/// Returns readiness evidence for the `WendaoGraph.jl` `PageIndex` reasoning
/// profile.
#[must_use]
pub fn wendao_graph_page_index_reasoning_readiness_evidence(
    input: WendaoGraphReadinessInput,
) -> JuliaReadinessEvidence {
    julia_static_contract_readiness_evidence(
        JuliaStaticContractReadinessProfile {
            capability: LaneCapability::GraphEvidenceCompute,
            profile_id: WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID,
            schema_version: WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION,
        },
        input.warmup,
        input.benchmark,
        input.window(),
    )
}

/// Returns a schedule plan for the `WendaoGraph.jl` `PageIndex` reasoning
/// profile.
#[must_use]
pub fn wendao_graph_page_index_reasoning_schedule_plan(
    shape: JuliaComputeTaskShape,
    facts: JuliaProfileSchedulingFacts,
) -> JuliaSchedulePlan {
    let readiness =
        wendao_graph_page_index_reasoning_readiness_evidence(WendaoGraphReadinessInput {
            warmup: facts.runtime_stats.warmup,
            benchmark: facts.runtime_stats.benchmark,
            max_in_flight: facts.max_in_flight,
            active_in_flight: facts.runtime_stats.active_in_flight,
            queue_depth: facts.runtime_stats.queue_depth,
        })
        .with_fallback_available(facts.fallback_available);
    julia_schedule_plan_from_readiness(readiness, shape, facts)
}

/// Returns runtime stats derived from a `WendaoGraph.jl` `PageIndex` host probe
/// report.
#[must_use]
pub fn wendao_graph_page_index_reasoning_runtime_stats_from_host_probe(
    report: &WendaoGraphPageIndexHostProbeReport,
) -> JuliaRuntimeStats {
    JuliaRuntimeStats::new()
        .with_warmup(WarmupState::Ready)
        .with_benchmark(BenchmarkState::NotRequired)
        .with_latency_ms(
            Some(latency_ms_as_u32(report.warm_median_ms)),
            Some(latency_ms_as_u32(report.warm_p95_ms)),
        )
}

/// Returns runtime stats derived from a `WendaoGraph.jl` `PageIndex`
/// planner-action host probe report.
#[must_use]
pub fn wendao_graph_page_index_reasoning_runtime_stats_from_planner_action_host_probe(
    report: &WendaoGraphPageIndexPlannerActionHostProbeReport,
) -> JuliaRuntimeStats {
    wendao_graph_page_index_reasoning_runtime_stats_from_host_probe(&report.base)
}

/// Returns readiness evidence derived from a `WendaoGraph.jl` `PageIndex` host
/// probe report.
#[must_use]
pub fn wendao_graph_page_index_reasoning_readiness_evidence_from_host_probe(
    report: &WendaoGraphPageIndexHostProbeReport,
    max_in_flight: Option<u32>,
    active_in_flight: u32,
    queue_depth: u32,
) -> JuliaReadinessEvidence {
    let benchmark = if report.sample_count > 0 {
        BenchmarkState::NotRequired
    } else {
        BenchmarkState::Failed
    };
    wendao_graph_page_index_reasoning_readiness_evidence(WendaoGraphReadinessInput {
        warmup: WarmupState::Ready,
        benchmark,
        max_in_flight,
        active_in_flight,
        queue_depth,
    })
}

/// Returns readiness evidence for the `WendaoGraph.jl` GNN reasoning profile.
#[must_use]
pub fn wendao_graph_gnn_reasoning_readiness_evidence(
    input: WendaoGraphReadinessInput,
) -> JuliaReadinessEvidence {
    julia_static_contract_readiness_evidence(
        JuliaStaticContractReadinessProfile {
            capability: LaneCapability::GraphEvidenceCompute,
            profile_id: WENDAO_GRAPH_GNN_REASONING_PROFILE_ID,
            schema_version: WENDAO_GRAPH_GNN_REASONING_SCHEMA_VERSION,
        },
        input.warmup,
        input.benchmark,
        input.window(),
    )
}

/// Returns runtime stats derived from a `WendaoGraph.jl` GNN host probe report.
#[must_use]
pub fn wendao_graph_gnn_runtime_stats_from_host_probe(
    report: &WendaoGraphGnnHostProbeReport,
) -> JuliaRuntimeStats {
    JuliaRuntimeStats::new()
        .with_warmup(WarmupState::Ready)
        .with_benchmark(BenchmarkState::NotRequired)
        .with_latency_ms(
            Some(latency_ms_as_u32(report.warm_median_ms)),
            Some(latency_ms_as_u32(report.warm_p95_ms)),
        )
}

/// Returns accelerator diagnostics derived from a `WendaoGraph.jl` GNN host
/// probe report.
#[must_use]
pub fn wendao_graph_gnn_accelerator_diagnostics_from_host_probe(
    report: &WendaoGraphGnnHostProbeReport,
) -> Vec<JuliaAcceleratorDiagnostics> {
    vec![
        JuliaAcceleratorDiagnostics::new(
            "metal",
            xiuxian_polyglot_orchestrator::JuliaAcceleratorState::new(
                xiuxian_polyglot_orchestrator::JuliaAcceleratorStateInput {
                    loaded: report.backend_load.metal_loaded,
                    functional: report.metal_functional,
                },
            ),
        )
        .with_observed_output_count(
            (report.metal_score_count > 0)
                .then_some(saturating_usize_to_u32(report.metal_score_count)),
        ),
        JuliaAcceleratorDiagnostics::new(
            "cuda",
            xiuxian_polyglot_orchestrator::JuliaAcceleratorState::new(
                xiuxian_polyglot_orchestrator::JuliaAcceleratorStateInput {
                    loaded: report.backend_load.cuda_loaded,
                    functional: false,
                },
            ),
        ),
        JuliaAcceleratorDiagnostics::new(
            "amdgpu",
            xiuxian_polyglot_orchestrator::JuliaAcceleratorState::new(
                xiuxian_polyglot_orchestrator::JuliaAcceleratorStateInput {
                    loaded: report.backend_load.amdgpu_loaded,
                    functional: false,
                },
            ),
        ),
    ]
}

/// Returns readiness evidence derived from a `WendaoGraph.jl` GNN host probe
/// report.
#[must_use]
pub fn wendao_graph_gnn_readiness_evidence_from_host_probe(
    report: &WendaoGraphGnnHostProbeReport,
    max_in_flight: Option<u32>,
    active_in_flight: u32,
    queue_depth: u32,
) -> JuliaReadinessEvidence {
    wendao_graph_gnn_reasoning_readiness_evidence(WendaoGraphReadinessInput {
        warmup: WarmupState::Ready,
        benchmark: BenchmarkState::NotRequired,
        max_in_flight,
        active_in_flight,
        queue_depth,
    })
    .with_accelerator_diagnostics(wendao_graph_gnn_accelerator_diagnostics_from_host_probe(
        report,
    ))
}

/// Returns a schedule plan for the `WendaoGraph.jl` GNN reasoning profile.
#[must_use]
pub fn wendao_graph_gnn_reasoning_schedule_plan(
    shape: JuliaComputeTaskShape,
    facts: JuliaProfileSchedulingFacts,
) -> JuliaSchedulePlan {
    let readiness = wendao_graph_gnn_reasoning_readiness_evidence(WendaoGraphReadinessInput {
        warmup: facts.runtime_stats.warmup,
        benchmark: facts.runtime_stats.benchmark,
        max_in_flight: facts.max_in_flight,
        active_in_flight: facts.runtime_stats.active_in_flight,
        queue_depth: facts.runtime_stats.queue_depth,
    })
    .with_fallback_available(facts.fallback_available);
    julia_schedule_plan_from_readiness(readiness, shape, facts)
}

/// Returns a schedule plan for one staged `WendaoGraph.jl` algorithm id.
///
/// Unknown algorithm ids return `None`; the caller can then choose an
/// owner-specific baseline or skip Julia for that request.
#[must_use]
pub fn wendaograph_algorithm_schedule_plan(
    algorithm_id: WendaoGraphAlgorithmId,
    workload: WendaoGraphAlgorithmWorkload,
    facts: JuliaProfileSchedulingFacts,
) -> Option<JuliaSchedulePlan> {
    let reference = wendaograph_algorithm_ref(algorithm_id)?;
    let shape = reference.task_shape(workload);
    match reference.profile_id {
        WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID => {
            Some(wendao_graph_link_evidence_schedule_plan(shape, facts))
        }
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID => Some(
            wendao_graph_page_index_reasoning_schedule_plan(shape, facts),
        ),
        WENDAO_GRAPH_GNN_REASONING_PROFILE_ID => {
            Some(wendao_graph_gnn_reasoning_schedule_plan(shape, facts))
        }
        _ => None,
    }
}

/// Returns a schedule plan for one reasoning-tree backend frontier evidence
/// kind.
///
/// Evidence kinds that remain Rust-owned, such as authority and negative-guard
/// checks, return `None`.
#[must_use]
pub fn wendaograph_frontier_schedule_plan(
    evidence_kind: &str,
    workload: WendaoGraphAlgorithmWorkload,
    facts: JuliaProfileSchedulingFacts,
) -> Option<JuliaSchedulePlan> {
    let reference = wendaograph_frontier_algorithm_ref(evidence_kind)?;
    wendaograph_algorithm_schedule_plan(
        WendaoGraphAlgorithmId(reference.algorithm_id),
        workload,
        facts,
    )
}

/// Projects every relationship-search algorithm into host-probe-backed
/// scheduling evidence.
#[must_use]
pub fn wendaograph_relationship_search_evidence_from_full_structural_host_probe(
    report: &WendaoGraphLinkGraphFullStructuralHostProbeReport,
    workload: WendaoGraphAlgorithmWorkload,
    facts: JuliaProfileSchedulingFacts,
) -> Vec<WendaoGraphRelationshipSearchEvidence> {
    wendaograph_relationship_search_algorithm_refs()
        .iter()
        .filter_map(|reference| {
            wendaograph_relationship_search_evidence_for_algorithm_from_full_structural_host_probe(
                WendaoGraphAlgorithmId(reference.algorithm_id),
                report,
                workload,
                facts,
            )
        })
        .collect()
}

/// Projects one relationship-search algorithm id into host-probe-backed
/// scheduling evidence.
///
/// Unknown ids, non-relationship-search ids, or ids that cannot route through
/// the existing algorithm schedule helper return `None`.
#[must_use]
pub fn wendaograph_relationship_search_evidence_for_algorithm_from_full_structural_host_probe(
    algorithm_id: WendaoGraphAlgorithmId,
    report: &WendaoGraphLinkGraphFullStructuralHostProbeReport,
    workload: WendaoGraphAlgorithmWorkload,
    facts: JuliaProfileSchedulingFacts,
) -> Option<WendaoGraphRelationshipSearchEvidence> {
    let algorithm = wendaograph_algorithm_ref(algorithm_id)?;
    if algorithm.family != "relationship_search" {
        return None;
    }

    let runtime_stats =
        relationship_search_runtime_stats_from_full_structural_host_probe(report, facts);
    let facts = JuliaProfileSchedulingFacts {
        runtime_stats,
        ..facts
    };
    let schedule_plan = wendaograph_algorithm_schedule_plan(algorithm_id, workload, facts)?;
    let (probe_table, probe_rows) = relationship_search_probe_rows(report, algorithm_id.0);
    Some(WendaoGraphRelationshipSearchEvidence {
        algorithm,
        probe_table,
        probe_rows,
        runtime_stats,
        schedule_plan,
    })
}

fn relationship_search_runtime_stats_from_full_structural_host_probe(
    report: &WendaoGraphLinkGraphFullStructuralHostProbeReport,
    facts: JuliaProfileSchedulingFacts,
) -> JuliaRuntimeStats {
    let probe_stats =
        wendao_graph_link_evidence_runtime_stats_from_full_structural_host_probe(report)
            .with_error_rate_basis_points(facts.runtime_stats.error_rate_basis_points)
            .with_queue(
                facts.runtime_stats.queue_depth,
                facts.runtime_stats.active_in_flight,
            );
    let benchmark = match facts.runtime_stats.benchmark {
        BenchmarkState::Unknown => probe_stats.benchmark,
        benchmark => benchmark,
    };
    probe_stats.with_benchmark(benchmark)
}

fn relationship_search_probe_rows(
    report: &WendaoGraphLinkGraphFullStructuralHostProbeReport,
    algorithm_id: &str,
) -> (Option<&'static str>, Option<u32>) {
    match algorithm_id {
        "relationship_search.hnsw_semantic_fanout"
        | "relationship_search.semantic_overlay_edges" => (
            Some("semantic_overlay"),
            Some(saturating_usize_to_u32(report.base.semantic_overlay_rows)),
        ),
        "relationship_search.moc_community_grouping" => (
            Some("topology_communities"),
            Some(saturating_usize_to_u32(report.topology_community_rows)),
        ),
        "relationship_search.community_bridge_links" => (
            Some("topology_community_links"),
            Some(saturating_usize_to_u32(report.topology_community_link_rows)),
        ),
        "relationship_search.community_frontier_ranking" => (
            Some("topology_community_frontier"),
            Some(saturating_usize_to_u32(
                report.topology_community_frontier_rows,
            )),
        ),
        "relationship_search.ppr_like_relatedness" => (
            Some("diffusion_scores"),
            Some(saturating_usize_to_u32(report.base.diffusion_rows)),
        ),
        "relationship_search.graph_search_ranking" => (
            Some("link_frontier"),
            Some(saturating_usize_to_u32(report.base.frontier_rows)),
        ),
        "relationship_search.topology_candidate_ranking" => (
            Some("topology_candidates"),
            Some(saturating_usize_to_u32(report.base.topology_candidate_rows)),
        ),
        "relationship_search.large_object_graph_traversal" => (
            Some("components"),
            Some(saturating_usize_to_u32(report.component_rows)),
        ),
        "relationship_search.graph_snapshot_traversal" => (
            Some("graph_metrics"),
            Some(saturating_usize_to_u32(report.base.graph_metric_rows)),
        ),
        _ => (None, None),
    }
}
