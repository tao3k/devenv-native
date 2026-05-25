//! Graph-structural Julia service fixtures for builtin integration tests.

use xiuxian_julia_core::integration_support::{
    JuliaServiceGuard, spawn_wendaosearch_solver_demo_multi_route_service,
    spawn_wendaosearch_solver_demo_structural_rerank_service,
};

pub use xiuxian_julia_core::{
    GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN, GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN,
    GRAPH_STRUCTURAL_CONSTRAINT_KIND_COLUMN, GRAPH_STRUCTURAL_DEPENDENCY_SCORE_COLUMN,
    GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN, GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN,
    GRAPH_STRUCTURAL_QUERY_ID_COLUMN, GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN,
    GRAPH_STRUCTURAL_REQUIRED_BOUNDARY_SIZE_COLUMN, GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN,
    GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN, GRAPH_STRUCTURAL_TAG_SCORE_COLUMN,
    GraphStructuralFilterConstraint, GraphStructuralFilterRequestRow,
    GraphStructuralFilterScoreRow, GraphStructuralRawConnectedPairCollectionCandidateInputs,
    GraphStructuralRerankScoreRow, build_graph_structural_filter_request_batch,
    build_graph_structural_generic_topology_candidate_inputs,
    build_graph_structural_generic_topology_candidate_inputs_from_pair_collection,
    build_graph_structural_generic_topology_candidate_inputs_from_raw_connected_pairs,
    build_graph_structural_generic_topology_candidate_inputs_from_scored_pair_collection,
    build_graph_structural_generic_topology_candidate_metadata_inputs,
    build_graph_structural_generic_topology_candidate_metadata_inputs_from_pair_collection,
    build_graph_structural_generic_topology_filter_request_batch,
    build_graph_structural_generic_topology_filter_request_batch_from_raw_connected_pair_collections,
    build_graph_structural_generic_topology_rerank_request_batch_from_raw_connected_pair_collections,
    build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_raw_candidates,
    build_graph_structural_keyword_overlap_query_inputs,
    build_graph_structural_keyword_overlap_raw_candidate_inputs,
    build_graph_structural_keyword_tag_query_context, build_graph_structural_pair_candidate_inputs,
    build_graph_structural_raw_connected_pair_collection_candidate_inputs,
    build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples,
    build_graph_structural_raw_connected_pair_inputs,
    build_graph_structural_scored_pair_candidate_inputs,
    fetch_graph_structural_filter_rows_for_repository,
    fetch_graph_structural_generic_topology_filter_rows_for_repository,
    fetch_graph_structural_generic_topology_filter_rows_for_repository_from_raw_connected_pair_collections,
    fetch_graph_structural_generic_topology_rerank_rows_for_repository,
    fetch_graph_structural_generic_topology_rerank_rows_for_repository_from_raw_connected_pair_collections,
    fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates,
};

/// Spawn the linked builtin single-route `WendaoSearch` `solver_demo`
/// structural-rerank service.
pub async fn linked_builtin_spawn_wendaosearch_solver_demo_structural_rerank_service()
-> (String, JuliaServiceGuard) {
    spawn_wendaosearch_solver_demo_structural_rerank_service().await
}

/// Spawn the linked builtin same-port multi-route `WendaoSearch`
/// `solver_demo` service.
pub async fn linked_builtin_spawn_wendaosearch_solver_demo_multi_route_service()
-> (String, JuliaServiceGuard) {
    spawn_wendaosearch_solver_demo_multi_route_service().await
}
