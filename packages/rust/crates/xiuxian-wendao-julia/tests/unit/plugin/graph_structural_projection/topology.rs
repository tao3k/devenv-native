//! Folder-first topology projection tests.

pub(super) use super::{
    GraphStructuralGenericTopologyCandidateInputs,
    GraphStructuralGenericTopologyCandidateMetadataInputs, GraphStructuralQueryAnchor,
    GraphStructuralQueryContext, build_graph_structural_generic_topology_candidate_inputs,
    build_graph_structural_generic_topology_candidate_inputs_from_pair_collection,
    build_graph_structural_generic_topology_candidate_inputs_from_raw_connected_pairs,
    build_graph_structural_generic_topology_candidate_inputs_from_scored_pair_collection,
    build_graph_structural_generic_topology_candidate_metadata_inputs,
    build_graph_structural_generic_topology_candidate_metadata_inputs_from_pair_collection,
    build_graph_structural_generic_topology_candidate_subgraph,
    build_graph_structural_generic_topology_rerank_request_batch,
    build_graph_structural_generic_topology_rerank_request_batch_from_raw_connected_pair_collections,
    build_graph_structural_generic_topology_rerank_request_row,
    build_graph_structural_keyword_tag_query_context, build_graph_structural_pair_candidate_inputs,
    build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples,
    build_graph_structural_raw_connected_pair_inputs,
    build_graph_structural_scored_pair_candidate_inputs,
};
pub(super) use crate::julia_plugin_test_support::common::{
    OptionTestExt, ResultTestExt, assert_f64_eq,
};
pub(super) use arrow::array::Float64Array;
pub(super) use arrow::array::StringArray;

mod batches;
mod candidate_rows;
mod collections;
