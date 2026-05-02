//! Graph-structural projection DTOs and row builders for Julia rerank routes.

mod core;
mod overlap;
mod overlap_builders;
mod pair;
mod pair_builders;
mod rows;
mod support;
mod topology;
mod topology_builders;

pub use core::{
    GraphStructuralCandidateSubgraph, GraphStructuralQueryAnchor, GraphStructuralQueryContext,
    GraphStructuralRerankSignals,
};
pub use overlap::{
    GraphStructuralFilterConstraint, GraphStructuralKeywordOverlapCandidateInputs,
    GraphStructuralKeywordOverlapPairInputs, GraphStructuralKeywordOverlapPairRequestInputs,
    GraphStructuralKeywordOverlapPairRerankInputs, GraphStructuralKeywordOverlapQueryInputs,
    GraphStructuralKeywordOverlapRawCandidateInputs, GraphStructuralNodeMetadataInputs,
    build_graph_structural_keyword_overlap_candidate_inputs,
    build_graph_structural_keyword_overlap_pair_candidate_inputs,
    build_graph_structural_keyword_overlap_pair_candidate_inputs_from_raw,
    build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs,
    build_graph_structural_keyword_overlap_query_inputs,
    build_graph_structural_keyword_overlap_raw_candidate_inputs,
    build_graph_structural_keyword_tag_query_context,
    build_graph_structural_keyword_tag_rerank_signals,
};
pub use overlap_builders::{
    build_graph_structural_keyword_overlap_pair_request_input,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_inputs,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_metadata,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_raw_candidates,
    build_graph_structural_keyword_overlap_pair_rerank_request_row,
    build_graph_structural_keyword_overlap_pair_rerank_request_row_from_metadata,
    graph_structural_shared_tag_anchors,
};
pub use pair::{
    GraphStructuralKeywordTagQueryInputs, GraphStructuralPairCandidateInputs,
    GraphStructuralRawConnectedPairInputs, GraphStructuralScoredPairCandidateInputs,
    build_graph_structural_pair_candidate_inputs, build_graph_structural_raw_connected_pair_inputs,
    build_graph_structural_scored_pair_candidate_inputs,
};
pub use pair_builders::{
    build_graph_structural_keyword_tag_pair_rerank_request_row,
    build_graph_structural_pair_candidate_subgraph, build_graph_structural_pair_filter_request_row,
    build_graph_structural_pair_rerank_request_row, graph_structural_pair_candidate_id,
};
pub use rows::{
    build_graph_structural_filter_request_row, build_graph_structural_rerank_request_row,
};
pub use topology::{
    GraphStructuralGenericTopologyCandidateInputs,
    GraphStructuralGenericTopologyCandidateMetadataInputs,
    GraphStructuralRawConnectedPairCollectionCandidateInputs,
    build_graph_structural_generic_topology_candidate_inputs,
    build_graph_structural_generic_topology_candidate_inputs_from_pair_collection,
    build_graph_structural_generic_topology_candidate_inputs_from_raw_connected_pairs,
    build_graph_structural_generic_topology_candidate_inputs_from_scored_pair_collection,
    build_graph_structural_generic_topology_candidate_metadata_inputs,
    build_graph_structural_generic_topology_candidate_metadata_inputs_from_pair_collection,
    build_graph_structural_raw_connected_pair_collection_candidate_inputs,
    build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples,
};
pub use topology_builders::{
    build_graph_structural_generic_topology_candidate_subgraph,
    build_graph_structural_generic_topology_filter_request_batch,
    build_graph_structural_generic_topology_filter_request_batch_from_raw_connected_pair_collections,
    build_graph_structural_generic_topology_filter_request_row,
    build_graph_structural_generic_topology_rerank_request_batch,
    build_graph_structural_generic_topology_rerank_request_batch_from_raw_connected_pair_collections,
    build_graph_structural_generic_topology_rerank_request_row,
};

#[cfg(test)]
#[path = "../../../tests/unit/plugin/graph_structural_projection/mod.rs"]
mod tests;
