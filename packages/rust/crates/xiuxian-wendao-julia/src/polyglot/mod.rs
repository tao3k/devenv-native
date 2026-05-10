//! Read-only projections from Julia-owned facts into polyglot contracts.

mod readiness;
mod state;
mod wendaograph_algorithms;

pub use crate::JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION;
pub use readiness::{
    memory_julia_compute_config_readiness, memory_julia_compute_readiness_evidence,
    memory_julia_compute_readiness_snapshot, memory_julia_compute_schedule_plan,
    memory_julia_compute_snapshot, wendao_graph_gnn_accelerator_diagnostics_from_host_probe,
    wendao_graph_gnn_readiness_evidence_from_host_probe,
    wendao_graph_gnn_reasoning_readiness_evidence, wendao_graph_gnn_reasoning_schedule_plan,
    wendao_graph_gnn_runtime_stats_from_host_probe, wendao_graph_link_evidence_readiness_evidence,
    wendao_graph_link_evidence_readiness_evidence_from_full_structural_host_probe,
    wendao_graph_link_evidence_runtime_stats_from_full_structural_host_probe,
    wendao_graph_link_evidence_runtime_stats_from_host_probe,
    wendao_graph_link_evidence_schedule_plan, wendao_graph_page_index_reasoning_readiness_evidence,
    wendao_graph_page_index_reasoning_readiness_evidence_from_host_probe,
    wendao_graph_page_index_reasoning_runtime_stats_from_host_probe,
    wendao_graph_page_index_reasoning_runtime_stats_from_planner_action_host_probe,
    wendao_graph_page_index_reasoning_schedule_plan, wendaograph_algorithm_schedule_plan,
    wendaograph_frontier_schedule_plan,
    wendaograph_relationship_search_evidence_for_algorithm_from_full_structural_host_probe,
    wendaograph_relationship_search_evidence_from_full_structural_host_probe,
    wendaosearch_graph_structural_readiness_evidence, wendaosearch_graph_structural_schedule_plan,
    wendaosearch_legacy_rerank_readiness_evidence, wendaosearch_legacy_rerank_schedule_plan,
    with_julia_thread_pinning_diagnostics,
};
pub use state::{
    JuliaProfileSchedulingFacts, WENDAO_GRAPH_GNN_REASONING_HOST_ENTRYPOINT,
    WENDAO_GRAPH_GNN_REASONING_PROFILE_ID, WENDAO_GRAPH_GNN_REASONING_SCHEMA_VERSION,
    WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID, WENDAO_GRAPH_PAGE_INDEX_REASONING_HOST_ENTRYPOINT,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID, WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID,
    WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID, WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID,
    WendaoGraphRelationshipSearchEvidence, julia_graph_compute_profile_refs,
    julia_graph_compute_snapshot, memory_julia_compute_manifest_row_ref,
    memory_julia_compute_profile_ref, memory_julia_compute_profile_refs,
    wendao_graph_gnn_reasoning_profile_ref, wendao_graph_link_evidence_profile_ref,
    wendao_graph_page_index_reasoning_profile_ref, wendaosearch_graph_structural_profile_ref,
    wendaosearch_graph_structural_profile_refs, wendaosearch_legacy_rerank_profile_ref,
};
pub use wendaograph_algorithms::{
    WendaoGraphAlgorithmRef, WendaoGraphAlgorithmWorkload, wendaograph_algorithm_ref,
    wendaograph_algorithm_refs, wendaograph_algorithm_refs_for_profile,
    wendaograph_algorithm_task_shape, wendaograph_frontier_algorithm_ref,
    wendaograph_frontier_task_shape, wendaograph_gnn_algorithm_refs,
    wendaograph_link_graph_algorithm_refs, wendaograph_page_index_algorithm_refs,
    wendaograph_relationship_search_algorithm_refs,
    wendaograph_search_strategy_flow_algorithm_refs,
};

#[cfg(test)]
#[path = "../../tests/unit/polyglot/mod.rs"]
mod tests;
