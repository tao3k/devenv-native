//! Readiness and scheduling projections for polyglot Julia profiles.

mod evidence_support;
mod graph;
mod memory;
mod wendaosearch;

pub use graph::{
    WendaoGraphReadinessInput, wendao_graph_gnn_accelerator_diagnostics_from_host_probe,
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
};
pub use memory::{
    MemoryJuliaComputeReadinessInput, memory_julia_compute_config_readiness,
    memory_julia_compute_readiness_evidence, memory_julia_compute_readiness_snapshot,
    memory_julia_compute_schedule_plan, memory_julia_compute_snapshot,
};
pub use wendaosearch::{
    WendaoSearchGraphStructuralReadinessInput, WendaoSearchLegacyRerankReadinessInput,
    wendaosearch_graph_structural_readiness_evidence, wendaosearch_graph_structural_schedule_plan,
    wendaosearch_legacy_rerank_readiness_evidence, wendaosearch_legacy_rerank_schedule_plan,
    with_julia_thread_pinning_diagnostics,
};
