//! Julia runtime fact projections for the polyglot control plane.

use super::facts as julia_wendao;
pub use super::facts::{
    MEMORY_JULIA_COMPUTE_CALIBRATION_PROFILE_ID,
    MEMORY_JULIA_COMPUTE_CALIBRATION_REQUEST_SCHEMA_ID,
    MEMORY_JULIA_COMPUTE_CALIBRATION_RESPONSE_SCHEMA_ID,
    MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_PROFILE_ID,
    MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_REQUEST_SCHEMA_ID,
    MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_RESPONSE_SCHEMA_ID, MEMORY_JULIA_COMPUTE_FAMILY_ID,
    MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID, MEMORY_JULIA_COMPUTE_GATE_SCORE_REQUEST_SCHEMA_ID,
    MEMORY_JULIA_COMPUTE_GATE_SCORE_RESPONSE_SCHEMA_ID,
    MEMORY_JULIA_COMPUTE_PLAN_TUNING_PROFILE_ID,
    MEMORY_JULIA_COMPUTE_PLAN_TUNING_REQUEST_SCHEMA_ID,
    MEMORY_JULIA_COMPUTE_PLAN_TUNING_RESPONSE_SCHEMA_ID, MemoryJuliaComputeProfile,
    WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION, WENDAO_GRAPH_GNN_REASONING_HOST_ENTRYPOINT,
    WENDAO_GRAPH_GNN_REASONING_PROFILE_ID, WENDAO_GRAPH_GNN_REASONING_SCHEMA_VERSION,
    WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID, WENDAO_GRAPH_LINK_EVIDENCE_ROUTE,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_HOST_ENTRYPOINT,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID, WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID,
    WENDAOSEARCH_CONSTRAINT_FILTER_ROUTE, WENDAOSEARCH_GRAPH_STRUCTURAL_SCHEMA_VERSION,
    WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID, WENDAOSEARCH_LEGACY_RERANK_ROUTE,
    WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID, WENDAOSEARCH_STRUCTURAL_RERANK_ROUTE,
    WendaoGraphAlgorithmComplexity, WendaoGraphAlgorithmId, WendaoGraphAlgorithmRef,
    WendaoGraphAlgorithmWorkload, WendaoGraphProfileId,
};

use crate::{
    ContractValidationState, JuliaComputeTaskShape, JuliaReadinessEvidence, JuliaRuntimeStats,
    JuliaSchedulePlan, JuliaSchedulingInput, JuliaTaskComplexityClass, LaneCapability,
    ManifestReadinessState,
};

/// Returns the Wendao-facing Julia runtime profiles known to the orchestrator.
#[must_use]
pub const fn wendao_julia_runtime_profile_ids() -> [&'static str; 6] {
    [
        WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID,
        WENDAO_GRAPH_GNN_REASONING_PROFILE_ID,
        WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID,
        WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID,
        WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID,
    ]
}

/// Scheduler-facing projection of one `WendaoGraph.jl` algorithm fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WendaoGraphScheduledAlgorithmRef {
    /// Stable Rust-facing algorithm id.
    pub algorithm_id: &'static str,
    /// Coarse algorithm family.
    pub family: &'static str,
    /// Julia profile that owns this algorithm.
    pub profile_id: &'static str,
    /// Julia function or host entrypoint that owns the implementation.
    pub julia_entrypoint: &'static str,
    /// Output table produced by the algorithm when it has a table surface.
    pub output_table: Option<&'static str>,
    /// Capability class used by the Rust scheduler.
    pub capability: LaneCapability,
    /// Owner-supplied scheduler complexity hint.
    pub complexity: JuliaTaskComplexityClass,
}

/// Owner-supplied scheduling facts for one Julia profile planning attempt.
///
/// These facts are inert. They do not start Julia, probe a worker, mutate a
/// queue, or execute fallback code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JuliaProfileSchedulingFacts {
    /// Optional maximum number of in-flight Julia requests for this profile.
    pub max_in_flight: Option<u32>,
    /// Runtime stats supplied by the owner package or host.
    pub runtime_stats: JuliaRuntimeStats,
    /// Whether an owner-defined fallback is safe for this task.
    pub fallback_available: bool,
    /// Optional hard deadline in milliseconds.
    pub deadline_ms: Option<u32>,
    /// Optional target latency in milliseconds.
    pub target_latency_ms: Option<u32>,
}

/// Relationship-search scheduling evidence projected from a `WendaoGraph.jl`
/// host probe.
///
/// This is descriptive owner evidence. It does not call Julia, add a route, or
/// turn row counts into hard admission gates.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoGraphRelationshipSearchEvidence {
    /// Catalog entry covered by this evidence row.
    pub algorithm: WendaoGraphScheduledAlgorithmRef,
    /// Host-probe table that backs this relationship-search algorithm row.
    pub probe_table: Option<&'static str>,
    /// Row count observed in the host-probe backing table.
    pub probe_rows: Option<u32>,
    /// Runtime stats projected from the host-probe timing report.
    pub runtime_stats: JuliaRuntimeStats,
    /// Schedule plan produced by the existing algorithm schedule helper.
    pub schedule_plan: JuliaSchedulePlan,
}

impl JuliaProfileSchedulingFacts {
    /// Creates scheduling facts from observed or inferred runtime stats.
    #[must_use]
    pub const fn new(runtime_stats: JuliaRuntimeStats) -> Self {
        Self {
            max_in_flight: None,
            runtime_stats,
            fallback_available: false,
            deadline_ms: None,
            target_latency_ms: None,
        }
    }

    /// Returns these facts with an admission capacity override.
    #[must_use]
    pub const fn with_max_in_flight(mut self, max_in_flight: Option<u32>) -> Self {
        self.max_in_flight = max_in_flight;
        self
    }

    /// Returns these facts with fallback availability.
    #[must_use]
    pub const fn with_fallback_available(mut self, fallback_available: bool) -> Self {
        self.fallback_available = fallback_available;
        self
    }

    /// Returns these facts with a hard deadline in milliseconds.
    #[must_use]
    pub const fn with_deadline_ms(mut self, deadline_ms: Option<u32>) -> Self {
        self.deadline_ms = deadline_ms;
        self
    }

    /// Returns these facts with a target latency in milliseconds.
    #[must_use]
    pub const fn with_target_latency_ms(mut self, target_latency_ms: Option<u32>) -> Self {
        self.target_latency_ms = target_latency_ms;
        self
    }
}

impl WendaoGraphScheduledAlgorithmRef {
    /// Projects a polyglot-owned Julia algorithm fact into scheduler terms.
    #[must_use]
    pub const fn from_runtime(reference: WendaoGraphAlgorithmRef) -> Self {
        Self {
            algorithm_id: reference.algorithm_id,
            family: reference.family,
            profile_id: reference.profile_id,
            julia_entrypoint: reference.julia_entrypoint,
            output_table: reference.output_table,
            capability: LaneCapability::GraphEvidenceCompute,
            complexity: complexity_from_runtime(reference.complexity),
        }
    }

    /// Returns whether this algorithm is marked as structurally heavy.
    #[must_use]
    pub const fn is_heavy(self) -> bool {
        matches!(self.complexity, JuliaTaskComplexityClass::Heavy)
    }

    /// Returns a scheduler task shape for this algorithm and workload.
    #[must_use]
    pub fn task_shape(self, workload: WendaoGraphAlgorithmWorkload) -> JuliaComputeTaskShape {
        JuliaComputeTaskShape::new()
            .with_rows(workload.rows.max(1))
            .with_graph_size(workload.nodes, workload.edges)
            .with_feature_columns(workload.feature_columns)
            .with_byte_size(workload.byte_size)
            .with_batchability_key(format!(
                "wendaograph:{}:{}",
                self.profile_id, self.algorithm_id
            ))
            .with_complexity(self.complexity)
    }
}

/// Returns the `WendaoGraph.jl` `LinkGraph` algorithm catalog in scheduler terms.
#[must_use]
pub fn wendaograph_link_graph_algorithm_refs() -> Vec<WendaoGraphScheduledAlgorithmRef> {
    project_algorithm_refs(julia_wendao::wendaograph_fact_link_graph_algorithm_refs())
}

/// Returns the `WendaoGraph.jl` relationship-search catalog in scheduler terms.
#[must_use]
pub fn wendaograph_relationship_search_algorithm_refs() -> Vec<WendaoGraphScheduledAlgorithmRef> {
    project_algorithm_refs(julia_wendao::wendaograph_fact_relationship_search_algorithm_refs())
}

/// Returns the `WendaoGraph.jl` `PageIndex` catalog in scheduler terms.
#[must_use]
pub fn wendaograph_page_index_algorithm_refs() -> Vec<WendaoGraphScheduledAlgorithmRef> {
    project_algorithm_refs(julia_wendao::wendaograph_fact_page_index_algorithm_refs())
}

/// Returns the `WendaoGraph.jl` `SearchStrategyFlow` catalog in scheduler terms.
#[must_use]
pub fn wendaograph_search_strategy_flow_algorithm_refs() -> Vec<WendaoGraphScheduledAlgorithmRef> {
    project_algorithm_refs(julia_wendao::wendaograph_fact_search_strategy_flow_algorithm_refs())
}

/// Returns the `WendaoGraph.jl` GNN catalog in scheduler terms.
#[must_use]
pub fn wendaograph_gnn_algorithm_refs() -> Vec<WendaoGraphScheduledAlgorithmRef> {
    project_algorithm_refs(julia_wendao::wendaograph_fact_gnn_algorithm_refs())
}

/// Returns all staged `WendaoGraph.jl` algorithms in scheduler terms.
#[must_use]
pub fn wendaograph_algorithm_refs() -> Vec<WendaoGraphScheduledAlgorithmRef> {
    julia_wendao::wendaograph_fact_algorithm_refs()
        .into_iter()
        .map(WendaoGraphScheduledAlgorithmRef::from_runtime)
        .collect()
}

/// Finds one staged `WendaoGraph.jl` algorithm by id in scheduler terms.
#[must_use]
pub fn wendaograph_algorithm_ref(
    algorithm_id: WendaoGraphAlgorithmId,
) -> Option<WendaoGraphScheduledAlgorithmRef> {
    julia_wendao::wendaograph_fact_algorithm_ref(algorithm_id)
        .map(WendaoGraphScheduledAlgorithmRef::from_runtime)
}

/// Returns the scheduler task shape for one staged `WendaoGraph.jl` algorithm.
#[must_use]
pub fn wendaograph_algorithm_task_shape(
    algorithm_id: WendaoGraphAlgorithmId,
    workload: WendaoGraphAlgorithmWorkload,
) -> Option<JuliaComputeTaskShape> {
    wendaograph_algorithm_ref(algorithm_id).map(|reference| reference.task_shape(workload))
}

/// Returns the staged `WendaoGraph.jl` algorithm for a frontier evidence kind.
#[must_use]
pub fn wendaograph_frontier_algorithm_ref(
    evidence_kind: &str,
) -> Option<WendaoGraphScheduledAlgorithmRef> {
    julia_wendao::wendaograph_fact_frontier_algorithm_ref(evidence_kind)
        .map(WendaoGraphScheduledAlgorithmRef::from_runtime)
}

/// Returns the scheduler task shape for one frontier evidence kind.
#[must_use]
pub fn wendaograph_frontier_task_shape(
    evidence_kind: &str,
    workload: WendaoGraphAlgorithmWorkload,
) -> Option<JuliaComputeTaskShape> {
    wendaograph_frontier_algorithm_ref(evidence_kind)
        .map(|reference| reference.task_shape(workload))
}

/// Returns a schedule plan for one staged `WendaoGraph.jl` algorithm id.
#[must_use]
pub fn wendaograph_algorithm_schedule_plan(
    algorithm_id: WendaoGraphAlgorithmId,
    workload: WendaoGraphAlgorithmWorkload,
    facts: JuliaProfileSchedulingFacts,
) -> Option<JuliaSchedulePlan> {
    let reference = wendaograph_algorithm_ref(algorithm_id)?;
    let shape = reference.task_shape(workload);
    Some(julia_schedule_plan_for_wendaograph_ref(
        reference, shape, facts,
    ))
}

/// Returns a schedule plan for one reasoning-tree backend frontier evidence kind.
#[must_use]
pub fn wendaograph_frontier_schedule_plan(
    evidence_kind: &str,
    workload: WendaoGraphAlgorithmWorkload,
    facts: JuliaProfileSchedulingFacts,
) -> Option<JuliaSchedulePlan> {
    let reference = wendaograph_frontier_algorithm_ref(evidence_kind)?;
    let shape = reference.task_shape(workload);
    Some(julia_schedule_plan_for_wendaograph_ref(
        reference, shape, facts,
    ))
}

/// Returns staged algorithms for one Julia profile id in scheduler terms.
#[must_use]
pub fn wendaograph_algorithm_refs_for_profile(
    profile_id: WendaoGraphProfileId,
) -> Vec<WendaoGraphScheduledAlgorithmRef> {
    julia_wendao::wendaograph_fact_algorithm_refs_for_profile(profile_id)
        .into_iter()
        .map(WendaoGraphScheduledAlgorithmRef::from_runtime)
        .collect()
}

fn project_algorithm_refs(
    references: &'static [WendaoGraphAlgorithmRef],
) -> Vec<WendaoGraphScheduledAlgorithmRef> {
    references
        .iter()
        .copied()
        .map(WendaoGraphScheduledAlgorithmRef::from_runtime)
        .collect()
}

const fn complexity_from_runtime(
    complexity: WendaoGraphAlgorithmComplexity,
) -> JuliaTaskComplexityClass {
    match complexity {
        WendaoGraphAlgorithmComplexity::Simple => JuliaTaskComplexityClass::Simple,
        WendaoGraphAlgorithmComplexity::Balanced => JuliaTaskComplexityClass::Balanced,
        WendaoGraphAlgorithmComplexity::Heavy => JuliaTaskComplexityClass::Heavy,
    }
}

fn julia_schedule_plan_for_wendaograph_ref(
    reference: WendaoGraphScheduledAlgorithmRef,
    shape: JuliaComputeTaskShape,
    facts: JuliaProfileSchedulingFacts,
) -> JuliaSchedulePlan {
    let readiness = JuliaReadinessEvidence::new(reference.capability, reference.profile_id)
        .with_schema_version(schema_version_for_wendaograph_profile(reference.profile_id))
        .with_route_validation(ContractValidationState::Valid)
        .with_schema_validation(ContractValidationState::Valid)
        .with_manifest_readiness(ManifestReadinessState::Ready)
        .with_warmup(facts.runtime_stats.warmup)
        .with_benchmark(facts.runtime_stats.benchmark)
        .with_admission_window(
            facts.max_in_flight,
            facts.runtime_stats.active_in_flight,
            facts.runtime_stats.queue_depth,
        )
        .with_fallback_available(facts.fallback_available);
    JuliaSchedulingInput::new(readiness, shape, facts.runtime_stats)
        .with_fallback_available(facts.fallback_available)
        .with_deadline_ms(facts.deadline_ms)
        .with_target_latency_ms(facts.target_latency_ms)
        .plan()
}

const fn schema_version_for_wendaograph_profile(profile_id: &str) -> &'static str {
    if str_eq(profile_id, WENDAO_GRAPH_GNN_REASONING_PROFILE_ID) {
        WENDAO_GRAPH_GNN_REASONING_SCHEMA_VERSION
    } else {
        WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION
    }
}

const fn str_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}
