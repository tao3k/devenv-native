//! Julia runtime facts consumed by the polyglot control plane.
//!
//! This module is intentionally projection-only. The runtime crate owns Julia
//! profile identities; the orchestrator consumes those facts when coordinating
//! cross-language work.

mod model;

pub use model::{
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
