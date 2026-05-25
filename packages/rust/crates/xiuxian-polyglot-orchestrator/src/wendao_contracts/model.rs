//! Read-only projections from Wendao runtime facts into polyglot contracts.

use xiuxian_wendao_runtime::config::{
    MemoryJuliaComputeFallbackMode, MemoryJuliaComputeRuntimeConfig,
};
use xiuxian_wendao_runtime::transport::ANALYSIS_DOCUMENT_EXTRACT_ROUTE;

use crate::{
    AdmissionBudget, DoclingSchedulePlan, DoclingSchedulingInput, PolyglotControlSnapshot,
    PolyglotLane, PressureLevel, ReadinessState, RouteProfileRef, SnapshotInvariantError,
    WorkerPressureEvidence,
};
#[cfg(feature = "julia-runtime")]
use crate::{
    ContractValidationState, FallbackEvidence, HealthState, JuliaComputeTaskShape,
    JuliaProfileSchedulingFacts, JuliaReadinessEvidence, JuliaSchedulePlan, JuliaSchedulingInput,
    LaneEvidence, LaneEvidenceInput, ManifestReadinessState, MemoryJuliaComputeProfile,
    WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION, WENDAO_GRAPH_GNN_REASONING_HOST_ENTRYPOINT,
    WENDAO_GRAPH_GNN_REASONING_PROFILE_ID, WENDAO_GRAPH_GNN_REASONING_SCHEMA_VERSION,
    WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID, WENDAO_GRAPH_LINK_EVIDENCE_ROUTE,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_HOST_ENTRYPOINT,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID, WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID,
    WENDAOSEARCH_CONSTRAINT_FILTER_ROUTE, WENDAOSEARCH_GRAPH_STRUCTURAL_SCHEMA_VERSION,
    WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID, WENDAOSEARCH_LEGACY_RERANK_ROUTE,
    WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID, WENDAOSEARCH_STRUCTURAL_RERANK_ROUTE,
};

/// Runtime facts used to project the Julia compute admission budget.
#[derive(Clone, Copy, Debug)]
pub struct MemoryJuliaComputeAdmissionBudgetInput<'a> {
    /// Runtime configuration resolved by the Wendao owner.
    pub config: &'a MemoryJuliaComputeRuntimeConfig,
    /// Currently active Julia compute requests.
    pub active_in_flight: u32,
    /// Queued Julia compute requests.
    pub queue_depth: u32,
    /// Current Julia readiness state.
    pub readiness: ReadinessState,
    /// Current Julia pressure level.
    pub pressure: PressureLevel,
}

/// Runtime facts used to project a neutral polyglot control snapshot.
#[derive(Clone, Copy, Debug)]
pub struct RuntimePolyglotSnapshotInput<'a> {
    /// Runtime configuration resolved by the Wendao owner.
    pub config: &'a MemoryJuliaComputeRuntimeConfig,
    /// Currently active Julia compute requests.
    pub active_in_flight: u32,
    /// Queued Julia compute requests.
    pub queue_depth: u32,
    /// Current Julia readiness state.
    pub readiness: ReadinessState,
    /// Current Julia pressure level.
    pub pressure: PressureLevel,
}

/// Owner-supplied pressure counters for document extraction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentExtractPressureEvidenceInput {
    /// Maximum in-flight worker count, when bounded.
    pub max_in_flight: Option<u32>,
    /// Currently active worker count.
    pub active_in_flight: u32,
    /// Queued shard count.
    pub queued_items: u32,
    /// Failed shard count.
    pub failed_items: u32,
    /// Retryable failure count.
    pub retryable_failures: u32,
    /// Whether the owner has a correctness-preserving fallback path.
    pub fallback_available: bool,
}

/// Runtime override facts for the legacy `WendaoSearch.jl` rerank route.
#[cfg(feature = "julia-runtime")]
#[derive(Clone, Copy, Debug, Default)]
pub struct WendaoSearchLegacyRerankProfileRefInput<'a> {
    /// Optional route override resolved by the runtime owner.
    pub route: Option<&'a str>,
    /// Optional schema-version override resolved by the runtime owner.
    pub schema_version: Option<&'a str>,
}

/// Readiness facts for one memory-family Julia compute profile.
#[cfg(feature = "julia-runtime")]
#[derive(Clone, Copy, Debug)]
pub struct MemoryJuliaComputeReadinessInput<'a> {
    /// Runtime config that owns route/schema/fallback facts.
    pub runtime: &'a MemoryJuliaComputeRuntimeConfig,
    /// Memory compute profile being described.
    pub profile: MemoryJuliaComputeProfile,
    /// Julia warmup state observed by the owner.
    pub warmup: crate::WarmupState,
    /// Benchmark state observed by the owner.
    pub benchmark: crate::BenchmarkState,
    /// Active in-flight request count.
    pub active_in_flight: u32,
    /// Queued request count.
    pub queue_depth: u32,
}

/// Returns a typed reference to the analyzer-owned document extraction route
/// known by Wendao runtime transport.
#[must_use]
pub fn document_extract_route_ref() -> RouteProfileRef {
    RouteProfileRef::document_extract(ANALYSIS_DOCUMENT_EXTRACT_ROUTE)
}

/// Returns typed refs for every staged memory-family Julia compute profile.
#[cfg(feature = "julia-runtime")]
#[must_use]
pub fn memory_julia_compute_profile_refs(
    runtime: &MemoryJuliaComputeRuntimeConfig,
) -> Vec<RouteProfileRef> {
    MemoryJuliaComputeProfile::ALL
        .into_iter()
        .map(|profile| memory_julia_compute_profile_ref(runtime, profile))
        .collect()
}

/// Returns a typed ref for one staged memory-family Julia compute profile.
#[cfg(feature = "julia-runtime")]
#[must_use]
pub fn memory_julia_compute_profile_ref(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    profile: MemoryJuliaComputeProfile,
) -> RouteProfileRef {
    RouteProfileRef::julia_profile(
        memory_route_for_profile(runtime, profile),
        profile.profile_id(),
        runtime.schema_version.as_str(),
    )
}

/// Builds a read-only snapshot for memory-family Julia compute contracts.
///
/// # Errors
///
/// Returns [`SnapshotInvariantError`] if the generated snapshot violates the
/// neutral orchestrator invariants.
#[cfg(feature = "julia-runtime")]
pub fn memory_julia_compute_snapshot(
    runtime: &MemoryJuliaComputeRuntimeConfig,
) -> Result<PolyglotControlSnapshot, SnapshotInvariantError> {
    PolyglotControlSnapshot::from_parts(
        memory_julia_compute_profile_refs(runtime),
        Vec::new(),
        vec![LaneEvidence::new(LaneEvidenceInput {
            lane: PolyglotLane::JuliaCompute,
            health: HealthState::Unknown,
            readiness: memory_julia_compute_config_readiness(runtime),
            pressure: PressureLevel::Unknown,
            fallback: FallbackEvidence::new(false),
        })],
    )
}

/// Returns Julia readiness evidence for one memory-family profile.
#[cfg(feature = "julia-runtime")]
#[must_use]
pub fn memory_julia_compute_readiness_evidence(
    input: MemoryJuliaComputeReadinessInput<'_>,
) -> JuliaReadinessEvidence {
    let MemoryJuliaComputeReadinessInput {
        runtime,
        profile,
        warmup,
        benchmark,
        active_in_flight,
        queue_depth,
    } = input;
    let schema_validation = if runtime.schema_version.is_empty() {
        ContractValidationState::Invalid
    } else {
        ContractValidationState::Valid
    };

    JuliaReadinessEvidence::memory_profile(profile.profile_id())
        .with_schema_version(runtime.schema_version.as_str())
        .with_route_validation(memory_route_validation(runtime, profile))
        .with_schema_validation(schema_validation)
        .with_manifest_readiness(memory_manifest_readiness(runtime))
        .with_warmup(warmup)
        .with_benchmark(benchmark)
        .with_admission_window(
            Some(max_in_flight_as_u32(runtime.max_in_flight_requests)),
            active_in_flight,
            queue_depth,
        )
        .with_fallback_available(matches!(
            runtime.fallback_mode,
            MemoryJuliaComputeFallbackMode::Rust
        ))
}

/// Returns a schedule plan for one memory-family Julia compute profile.
#[cfg(feature = "julia-runtime")]
#[must_use]
pub fn memory_julia_compute_schedule_plan(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    profile: MemoryJuliaComputeProfile,
    shape: JuliaComputeTaskShape,
    facts: JuliaProfileSchedulingFacts,
) -> JuliaSchedulePlan {
    let fallback_available = facts.fallback_available
        || matches!(runtime.fallback_mode, MemoryJuliaComputeFallbackMode::Rust);
    let facts = facts.with_fallback_available(fallback_available);
    let readiness = memory_julia_compute_readiness_evidence(MemoryJuliaComputeReadinessInput {
        runtime,
        profile,
        warmup: facts.runtime_stats.warmup,
        benchmark: facts.runtime_stats.benchmark,
        active_in_flight: facts.runtime_stats.active_in_flight,
        queue_depth: facts.runtime_stats.queue_depth,
    })
    .with_fallback_available(fallback_available);
    JuliaSchedulingInput::new(readiness, shape, facts.runtime_stats)
        .with_fallback_available(facts.fallback_available)
        .with_deadline_ms(facts.deadline_ms)
        .with_target_latency_ms(facts.target_latency_ms)
        .plan()
}

/// Builds a read-only snapshot for one memory-family Julia readiness profile.
///
/// # Errors
///
/// Returns [`SnapshotInvariantError`] if the generated snapshot violates the
/// neutral orchestrator invariants.
#[cfg(feature = "julia-runtime")]
pub fn memory_julia_compute_readiness_snapshot(
    input: MemoryJuliaComputeReadinessInput<'_>,
) -> Result<PolyglotControlSnapshot, SnapshotInvariantError> {
    let runtime = input.runtime;
    let profile = input.profile;
    let evidence = memory_julia_compute_readiness_evidence(input);
    PolyglotControlSnapshot::from_parts(
        vec![memory_julia_compute_profile_ref(runtime, profile)],
        vec![evidence.to_admission_budget()],
        vec![evidence.to_lane_evidence()],
    )
}

/// Returns config-level readiness for the memory-family Julia compute lane.
#[cfg(feature = "julia-runtime")]
#[must_use]
pub const fn memory_julia_compute_config_readiness(
    runtime: &MemoryJuliaComputeRuntimeConfig,
) -> ReadinessState {
    if runtime.enabled {
        ReadinessState::Ready
    } else {
        ReadinessState::Disabled
    }
}

/// Returns the typed ref for the `WendaoGraph.jl` link-evidence contract.
#[cfg(feature = "julia-runtime")]
#[must_use]
pub fn wendao_graph_link_evidence_profile_ref() -> RouteProfileRef {
    RouteProfileRef::julia_profile(
        WENDAO_GRAPH_LINK_EVIDENCE_ROUTE,
        WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
        WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION,
    )
}

/// Returns the typed ref for the `WendaoGraph.jl` `PageIndex` reasoning contract.
#[cfg(feature = "julia-runtime")]
#[must_use]
pub fn wendao_graph_page_index_reasoning_profile_ref() -> RouteProfileRef {
    RouteProfileRef::julia_profile(
        WENDAO_GRAPH_PAGE_INDEX_REASONING_HOST_ENTRYPOINT,
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID,
        WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION,
    )
}

/// Returns the typed ref for the `WendaoGraph.jl` GNN reasoning contract.
#[cfg(feature = "julia-runtime")]
#[must_use]
pub fn wendao_graph_gnn_reasoning_profile_ref() -> RouteProfileRef {
    RouteProfileRef::julia_profile(
        WENDAO_GRAPH_GNN_REASONING_HOST_ENTRYPOINT,
        WENDAO_GRAPH_GNN_REASONING_PROFILE_ID,
        WENDAO_GRAPH_GNN_REASONING_SCHEMA_VERSION,
    )
}

/// Returns a typed ref for the `WendaoSearch.jl` structural-rerank route.
#[cfg(feature = "julia-runtime")]
#[must_use]
pub fn wendaosearch_structural_rerank_profile_ref() -> RouteProfileRef {
    RouteProfileRef::julia_profile(
        WENDAOSEARCH_STRUCTURAL_RERANK_ROUTE,
        WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID,
        WENDAOSEARCH_GRAPH_STRUCTURAL_SCHEMA_VERSION,
    )
}

/// Returns a typed ref for the `WendaoSearch.jl` constraint-filter route.
#[cfg(feature = "julia-runtime")]
#[must_use]
pub fn wendaosearch_constraint_filter_profile_ref() -> RouteProfileRef {
    RouteProfileRef::julia_profile(
        WENDAOSEARCH_CONSTRAINT_FILTER_ROUTE,
        WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID,
        WENDAOSEARCH_GRAPH_STRUCTURAL_SCHEMA_VERSION,
    )
}

/// Returns typed refs for the staged `WendaoSearch.jl` graph-structural routes.
#[cfg(feature = "julia-runtime")]
#[must_use]
pub fn wendaosearch_graph_structural_profile_refs() -> Vec<RouteProfileRef> {
    vec![
        wendaosearch_structural_rerank_profile_ref(),
        wendaosearch_constraint_filter_profile_ref(),
    ]
}

/// Returns the typed ref for the legacy `WendaoSearch.jl` rerank route.
#[cfg(feature = "julia-runtime")]
#[must_use]
pub fn wendaosearch_legacy_rerank_profile_ref(
    input: WendaoSearchLegacyRerankProfileRefInput<'_>,
) -> RouteProfileRef {
    RouteProfileRef::julia_profile(
        input.route.unwrap_or(WENDAOSEARCH_LEGACY_RERANK_ROUTE),
        WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID,
        input.schema_version.unwrap_or("v1"),
    )
}

/// Returns graph-family Julia route refs currently known to the Rust scheduler.
#[cfg(feature = "julia-runtime")]
#[must_use]
pub fn julia_graph_compute_profile_refs(
    legacy_rerank: WendaoSearchLegacyRerankProfileRefInput<'_>,
) -> Vec<RouteProfileRef> {
    let mut refs = Vec::with_capacity(6);
    refs.push(wendao_graph_link_evidence_profile_ref());
    refs.push(wendao_graph_page_index_reasoning_profile_ref());
    refs.push(wendao_graph_gnn_reasoning_profile_ref());
    refs.push(wendaosearch_legacy_rerank_profile_ref(legacy_rerank));
    refs.extend(wendaosearch_graph_structural_profile_refs());
    refs
}

/// Returns the Wendao runtime-owned Julia compute admission budget.
#[must_use]
pub fn memory_julia_compute_admission_budget(
    input: MemoryJuliaComputeAdmissionBudgetInput<'_>,
) -> AdmissionBudget {
    AdmissionBudget {
        lane: PolyglotLane::JuliaCompute,
        max_in_flight: Some(max_in_flight_as_u32(input.config.max_in_flight_requests)),
        active_in_flight: input.active_in_flight,
        queue_depth: input.queue_depth,
        readiness: input.readiness,
        pressure: input.pressure,
        fallback_available: matches!(
            input.config.fallback_mode,
            MemoryJuliaComputeFallbackMode::Rust
        ),
    }
}

/// Builds a read-only runtime control-plane snapshot from already supplied
/// Wendao runtime facts.
///
/// # Errors
///
/// Returns [`SnapshotInvariantError`] if the generated snapshot violates the
/// neutral orchestrator invariants.
pub fn runtime_polyglot_snapshot(
    input: RuntimePolyglotSnapshotInput<'_>,
) -> Result<PolyglotControlSnapshot, SnapshotInvariantError> {
    PolyglotControlSnapshot::from_parts(
        vec![document_extract_route_ref()],
        vec![memory_julia_compute_admission_budget(
            MemoryJuliaComputeAdmissionBudgetInput {
                config: input.config,
                active_in_flight: input.active_in_flight,
                queue_depth: input.queue_depth,
                readiness: input.readiness,
                pressure: input.pressure,
            },
        )],
        Vec::new(),
    )
}

/// Returns document extraction pressure evidence from owner-supplied counters.
#[must_use]
pub fn document_extract_pressure_evidence(
    input: DocumentExtractPressureEvidenceInput,
) -> WorkerPressureEvidence {
    WorkerPressureEvidence::document_extraction()
        .with_worker_budget(input.max_in_flight, input.active_in_flight)
        .with_queue_depth(input.queued_items)
        .with_failures(input.failed_items, input.retryable_failures)
        .with_fallback_available(input.fallback_available)
}

/// Builds a read-only document extraction pressure snapshot from supplied
/// counters.
///
/// # Errors
///
/// Returns [`SnapshotInvariantError`] if the generated snapshot violates the
/// neutral orchestrator invariants.
pub fn document_extract_pressure_snapshot(
    pressure: WorkerPressureEvidence,
) -> Result<PolyglotControlSnapshot, SnapshotInvariantError> {
    PolyglotControlSnapshot::from_parts(
        vec![document_extract_route_ref()],
        vec![pressure.to_admission_budget()],
        vec![pressure.to_lane_evidence()],
    )
}

/// Returns an inert document extraction scheduling plan from runtime-owned
/// pressure facts and caller-local worker bounds.
#[must_use]
pub fn document_extract_schedule_plan(
    pressure: WorkerPressureEvidence,
    requested_workers: Option<u32>,
    max_worker_cap: Option<u32>,
    shard_count: u32,
) -> DoclingSchedulePlan {
    DoclingSchedulingInput::document_extraction(pressure)
        .with_worker_request(requested_workers, max_worker_cap)
        .with_shard_count(shard_count)
        .plan()
}

fn max_in_flight_as_u32(max_in_flight_requests: u64) -> u32 {
    u32::try_from(max_in_flight_requests).unwrap_or(u32::MAX)
}

#[cfg(feature = "julia-runtime")]
fn memory_route_for_profile(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    profile: MemoryJuliaComputeProfile,
) -> &str {
    match profile {
        MemoryJuliaComputeProfile::EpisodicRecall => runtime.routes.episodic_recall.as_str(),
        MemoryJuliaComputeProfile::MemoryGateScore => runtime.routes.memory_gate_score.as_str(),
        MemoryJuliaComputeProfile::MemoryPlanTuning => runtime.routes.memory_plan_tuning.as_str(),
        MemoryJuliaComputeProfile::MemoryCalibration => runtime.routes.memory_calibration.as_str(),
    }
}

#[cfg(feature = "julia-runtime")]
const fn memory_manifest_readiness(
    runtime: &MemoryJuliaComputeRuntimeConfig,
) -> ManifestReadinessState {
    if runtime.enabled {
        ManifestReadinessState::Ready
    } else {
        ManifestReadinessState::Missing
    }
}

#[cfg(feature = "julia-runtime")]
fn memory_route_validation(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    profile: MemoryJuliaComputeProfile,
) -> ContractValidationState {
    if memory_route_for_profile(runtime, profile).is_empty() {
        ContractValidationState::Invalid
    } else {
        ContractValidationState::Valid
    }
}
