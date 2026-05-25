//! Wendao-facing Julia runtime contract facts.
//!
//! The types in this module are inert identities and workload descriptors. They
//! do not start Julia, open Flight transports, or schedule work by themselves.

mod catalog;
mod memory_binding;
mod memory_profile;
mod profile;
mod workload;

pub use catalog::{
    WendaoGraphAlgorithmComplexity, WendaoGraphAlgorithmRef, wendaograph_algorithm_ref,
    wendaograph_algorithm_refs, wendaograph_algorithm_refs_for_profile,
    wendaograph_frontier_algorithm_ref, wendaograph_gnn_algorithm_refs,
    wendaograph_link_graph_algorithm_refs, wendaograph_page_index_algorithm_refs,
    wendaograph_relationship_search_algorithm_refs,
    wendaograph_search_strategy_flow_algorithm_refs,
};
pub use memory_binding::{build_memory_julia_compute_binding, build_memory_julia_compute_bindings};
pub use memory_profile::{
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
};
pub use profile::{
    WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION, WENDAO_GRAPH_GNN_REASONING_HOST_ENTRYPOINT,
    WENDAO_GRAPH_GNN_REASONING_PROFILE_ID, WENDAO_GRAPH_GNN_REASONING_SCHEMA_VERSION,
    WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID, WENDAO_GRAPH_LINK_EVIDENCE_ROUTE,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_HOST_ENTRYPOINT,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID, WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID,
    WENDAOSEARCH_CONSTRAINT_FILTER_ROUTE, WENDAOSEARCH_GRAPH_STRUCTURAL_SCHEMA_VERSION,
    WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID, WENDAOSEARCH_LEGACY_RERANK_ROUTE,
    WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID, WENDAOSEARCH_STRUCTURAL_RERANK_ROUTE,
    WendaoGraphAlgorithmId, WendaoGraphProfileId,
};
pub use workload::WendaoGraphAlgorithmWorkload;
