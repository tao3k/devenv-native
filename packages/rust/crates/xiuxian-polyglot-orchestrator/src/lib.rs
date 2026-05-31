//! Thin polyglot control-plane contracts for Wendao compute lanes.
//!
//! This crate names shared contracts only. Execution ownership stays in the
//! existing runtime, attachments, analyzer, and Julia packages.

#[cfg(test)]
#[path = "../tests/unit/lib/mod.rs"]
mod tests;

/// Admission budget and decision contracts.
pub mod admission;
/// Pure scheduling contracts for Python audio shard work.
pub mod audio_schedule;
/// Pure scheduling contracts for Python Docling work.
pub mod docling_schedule;
/// Health, readiness, pressure, and fallback evidence contracts.
pub mod evidence;
#[cfg(feature = "julia-runtime")]
/// Julia facts consumed by Julia core/runtime through the polyglot control plane.
pub mod julia_runtime;
/// Pure scheduling contracts for Julia compute profiles.
pub mod julia_schedule;
/// Lane identity and capability classification.
pub mod lanes;
/// Worker pressure evidence contracts.
pub mod pressure;
/// Julia readiness evidence contracts.
pub mod readiness;
/// Typed references to external owner contracts.
pub mod refs;
/// Schema benchmark evidence contracts.
pub mod schema_benchmark;
/// Read-only control-plane snapshots.
pub mod snapshot;
#[cfg(feature = "wendao-contracts")]
/// Wendao-owned runtime facts projected into polyglot contracts.
pub mod wendao_contracts;

pub use admission::{AdmissionBudget, AdmissionDecision, QueueReason, RejectionReason};
pub use audio_schedule::{
    AudioScheduleAction, AudioSchedulePlan, AudioScheduleReason, AudioSchedulingInput,
};
pub use docling_schedule::{
    DoclingScheduleAction, DoclingSchedulePlan, DoclingScheduleReason, DoclingSchedulingInput,
    DoclingWorkerPolicy,
};
pub use evidence::{
    FallbackEvidence, HealthState, LaneEvidence, LaneEvidenceInput, PressureLevel, ReadinessState,
};
#[cfg(feature = "julia-runtime")]
pub use julia_runtime::{
    JuliaProfileSchedulingFacts, MEMORY_JULIA_COMPUTE_CALIBRATION_PROFILE_ID,
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
    WendaoGraphAlgorithmWorkload, WendaoGraphProfileId, WendaoGraphRelationshipSearchEvidence,
    WendaoGraphScheduledAlgorithmRef, wendao_julia_runtime_profile_ids, wendaograph_algorithm_ref,
    wendaograph_algorithm_refs, wendaograph_algorithm_refs_for_profile,
    wendaograph_algorithm_schedule_plan, wendaograph_algorithm_task_shape,
    wendaograph_frontier_algorithm_ref, wendaograph_frontier_schedule_plan,
    wendaograph_frontier_task_shape, wendaograph_gnn_algorithm_refs,
    wendaograph_link_graph_algorithm_refs, wendaograph_page_index_algorithm_refs,
    wendaograph_relationship_search_algorithm_refs,
    wendaograph_search_strategy_flow_algorithm_refs,
};
pub use julia_schedule::{
    JuliaComputeTaskShape, JuliaRuntimeStats, JuliaScheduleAction, JuliaScheduleBatchabilityKey,
    JuliaScheduleLatencyMs, JuliaSchedulePlan, JuliaScheduleProfileId, JuliaScheduleReason,
    JuliaSchedulingInput, JuliaTaskComplexityClass,
};
pub use lanes::{LaneCapability, PolyglotLane};
pub use pressure::WorkerPressureEvidence;
pub use readiness::{
    BenchmarkState, ContractValidationState, JuliaAcceleratorDiagnostics, JuliaAcceleratorState,
    JuliaAcceleratorStateInput, JuliaReadinessEvidence, JuliaThreadPinningDiagnostics,
    JuliaThreadPinningState, JuliaThreadTopology, ManifestReadinessState, WarmupState,
};
pub use refs::{ContractOwner, RouteProfileRef};
pub use schema_benchmark::{
    CachePressureBytes, EncodedByteSize, MemoryPressureBytes, SchemaBenchmarkCase,
    SchemaBenchmarkEvidence, SchemaBenchmarkReport, SchemaBenchmarkReportError,
    SchemaStrategyCandidate, SchemaStrategyPreference,
};
pub use snapshot::{PolyglotControlSnapshot, SnapshotInvariantError};
#[cfg(feature = "wendao-contracts")]
pub use wendao_contracts::{
    DocumentExtractPressureEvidenceInput, MemoryJuliaComputeAdmissionBudgetInput,
    RuntimePolyglotSnapshotInput, document_extract_pressure_evidence,
    document_extract_pressure_snapshot, document_extract_route_ref, document_extract_schedule_plan,
    memory_julia_compute_admission_budget, runtime_polyglot_snapshot,
};
#[cfg(all(feature = "wendao-contracts", feature = "julia-runtime"))]
pub use wendao_contracts::{
    MemoryJuliaComputeReadinessInput, WendaoSearchLegacyRerankProfileRefInput,
    julia_graph_compute_profile_refs, memory_julia_compute_config_readiness,
    memory_julia_compute_profile_ref, memory_julia_compute_profile_refs,
    memory_julia_compute_readiness_evidence, memory_julia_compute_readiness_snapshot,
    memory_julia_compute_schedule_plan, memory_julia_compute_snapshot,
    wendao_graph_gnn_reasoning_profile_ref, wendao_graph_link_evidence_profile_ref,
    wendao_graph_page_index_reasoning_profile_ref, wendaosearch_constraint_filter_profile_ref,
    wendaosearch_graph_structural_profile_refs, wendaosearch_legacy_rerank_profile_ref,
    wendaosearch_structural_rerank_profile_ref,
};
