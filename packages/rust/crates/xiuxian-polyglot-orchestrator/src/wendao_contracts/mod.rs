//! Wendao runtime fact projections into polyglot contracts.

mod model;

pub use model::{
    DocumentExtractPressureEvidenceInput, MemoryJuliaComputeAdmissionBudgetInput,
    RuntimePolyglotSnapshotInput, document_extract_pressure_evidence,
    document_extract_pressure_snapshot, document_extract_route_ref, document_extract_schedule_plan,
    memory_julia_compute_admission_budget, runtime_polyglot_snapshot,
};
#[cfg(feature = "julia-runtime")]
pub use model::{
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
