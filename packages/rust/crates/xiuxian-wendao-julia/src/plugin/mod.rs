//! Julia repository intelligence plugin contracts and transport owners.

mod capability_manifest;
mod discovery;
mod entry;
mod graph_structural;
pub(crate) mod graph_structural_exchange;
mod graph_structural_projection;
mod graph_structural_transport;
mod linking;
/// Parser-summary contract and transport helpers for Julia source analysis.
pub mod parser_summary;
mod project;
mod rerank_exchange;
mod sources;
mod transport;
mod wendao_graph_evidence;

#[cfg(test)]
#[path = "../../tests/unit/plugin/mod.rs"]
pub(crate) mod test_support;

pub use capability_manifest::{
    JULIA_PLUGIN_CAPABILITY_MANIFEST_BASE_URL_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_FILTER_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_ENABLED_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_HEALTH_ROUTE_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_INCLUDE_DISABLED_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_PLUGIN_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_REPOSITORY_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_REQUEST_COLUMNS,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_COLUMNS,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_PLUGIN_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE, JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_TRANSPORT_KIND_COLUMN,
    JuliaPluginCapabilityManifestRequestRow, JuliaPluginCapabilityManifestRow,
    build_julia_capability_manifest_flight_transport_client,
    build_julia_plugin_capability_manifest_request_batch,
    decode_julia_plugin_capability_manifest_rows,
    fetch_julia_plugin_capability_manifest_rows_for_repository,
    process_julia_capability_manifest_flight_batches,
    process_julia_capability_manifest_flight_batches_for_repository,
    validate_julia_plugin_capability_manifest_request_batches,
    validate_julia_plugin_capability_manifest_response_batches,
};
pub use entry::JuliaRepoIntelligencePlugin;
pub use entry::register_into;
pub use graph_structural::{
    GRAPH_STRUCTURAL_ACCEPTED_COLUMN, GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN,
    GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN,
    GRAPH_STRUCTURAL_CONSTRAINT_KIND_COLUMN, GRAPH_STRUCTURAL_DEPENDENCY_SCORE_COLUMN,
    GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN, GRAPH_STRUCTURAL_EXPLANATION_COLUMN,
    GRAPH_STRUCTURAL_FEASIBLE_COLUMN, GRAPH_STRUCTURAL_FILTER_REQUEST_COLUMNS,
    GRAPH_STRUCTURAL_FILTER_RESPONSE_COLUMNS, GRAPH_STRUCTURAL_FILTER_ROUTE,
    GRAPH_STRUCTURAL_FINAL_SCORE_COLUMN, GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN,
    GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN, GRAPH_STRUCTURAL_QUERY_ID_COLUMN,
    GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN, GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN,
    GRAPH_STRUCTURAL_REQUIRED_BOUNDARY_SIZE_COLUMN, GRAPH_STRUCTURAL_RERANK_REQUEST_COLUMNS,
    GRAPH_STRUCTURAL_RERANK_RESPONSE_COLUMNS, GRAPH_STRUCTURAL_RERANK_ROUTE,
    GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN, GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN,
    GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN, GRAPH_STRUCTURAL_TAG_SCORE_COLUMN,
    GraphStructuralRouteKind, JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION, graph_structural_route_kind,
    is_graph_structural_route, validate_graph_structural_filter_request_batch,
    validate_graph_structural_filter_request_schema,
    validate_graph_structural_filter_response_batch,
    validate_graph_structural_filter_response_schema,
    validate_graph_structural_rerank_request_batch,
    validate_graph_structural_rerank_request_schema,
    validate_graph_structural_rerank_response_batch,
    validate_graph_structural_rerank_response_schema,
};
pub use graph_structural_exchange::{
    GraphStructuralFilterRequestRow, GraphStructuralFilterScoreRow,
    GraphStructuralRerankRequestRow, GraphStructuralRerankScoreRow,
    build_graph_structural_filter_request_batch, build_graph_structural_rerank_request_batch,
    decode_graph_structural_filter_score_rows, decode_graph_structural_rerank_score_rows,
    fetch_graph_structural_filter_rows_for_repository,
    fetch_graph_structural_generic_topology_filter_rows_for_repository,
    fetch_graph_structural_generic_topology_filter_rows_for_repository_from_raw_connected_pair_collections,
    fetch_graph_structural_generic_topology_rerank_rows_for_repository,
    fetch_graph_structural_generic_topology_rerank_rows_for_repository_from_raw_connected_pair_collections,
    fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository,
    fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates,
    fetch_graph_structural_rerank_rows_for_repository,
};
pub use graph_structural_projection::{
    GraphStructuralCandidateSubgraph, GraphStructuralFilterConstraint,
    GraphStructuralGenericTopologyCandidateInputs,
    GraphStructuralGenericTopologyCandidateMetadataInputs,
    GraphStructuralKeywordOverlapCandidateInputs, GraphStructuralKeywordOverlapPairInputs,
    GraphStructuralKeywordOverlapPairRequestInputs, GraphStructuralKeywordOverlapPairRerankInputs,
    GraphStructuralKeywordOverlapQueryInputs, GraphStructuralKeywordOverlapRawCandidateInputs,
    GraphStructuralKeywordTagQueryInputs, GraphStructuralNodeMetadataInputs,
    GraphStructuralPairCandidateInputs, GraphStructuralQueryAnchor, GraphStructuralQueryContext,
    GraphStructuralRawConnectedPairCollectionCandidateInputs,
    GraphStructuralRawConnectedPairInputs, GraphStructuralRerankSignals,
    GraphStructuralScoredPairCandidateInputs, build_graph_structural_filter_request_row,
    build_graph_structural_generic_topology_candidate_inputs,
    build_graph_structural_generic_topology_candidate_inputs_from_pair_collection,
    build_graph_structural_generic_topology_candidate_inputs_from_raw_connected_pairs,
    build_graph_structural_generic_topology_candidate_inputs_from_scored_pair_collection,
    build_graph_structural_generic_topology_candidate_metadata_inputs,
    build_graph_structural_generic_topology_candidate_metadata_inputs_from_pair_collection,
    build_graph_structural_generic_topology_candidate_subgraph,
    build_graph_structural_generic_topology_filter_request_batch,
    build_graph_structural_generic_topology_filter_request_batch_from_raw_connected_pair_collections,
    build_graph_structural_generic_topology_filter_request_row,
    build_graph_structural_generic_topology_rerank_request_batch,
    build_graph_structural_generic_topology_rerank_request_batch_from_raw_connected_pair_collections,
    build_graph_structural_generic_topology_rerank_request_row,
    build_graph_structural_keyword_overlap_candidate_inputs,
    build_graph_structural_keyword_overlap_pair_candidate_inputs,
    build_graph_structural_keyword_overlap_pair_candidate_inputs_from_raw,
    build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs,
    build_graph_structural_keyword_overlap_pair_request_input,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_inputs,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_metadata,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_raw_candidates,
    build_graph_structural_keyword_overlap_pair_rerank_request_row,
    build_graph_structural_keyword_overlap_pair_rerank_request_row_from_metadata,
    build_graph_structural_keyword_overlap_query_inputs,
    build_graph_structural_keyword_overlap_raw_candidate_inputs,
    build_graph_structural_keyword_tag_pair_rerank_request_row,
    build_graph_structural_keyword_tag_query_context,
    build_graph_structural_keyword_tag_rerank_signals,
    build_graph_structural_pair_candidate_inputs, build_graph_structural_pair_candidate_subgraph,
    build_graph_structural_pair_filter_request_row, build_graph_structural_pair_rerank_request_row,
    build_graph_structural_raw_connected_pair_collection_candidate_inputs,
    build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples,
    build_graph_structural_raw_connected_pair_inputs, build_graph_structural_rerank_request_row,
    build_graph_structural_scored_pair_candidate_inputs, graph_structural_pair_candidate_id,
    graph_structural_shared_tag_anchors,
};
pub use graph_structural_transport::{
    build_graph_structural_flight_transport_client, process_graph_structural_flight_batches,
    process_graph_structural_flight_batches_for_repository,
    validate_graph_structural_request_batches, validate_graph_structural_response_batches,
};
pub use parser_summary::JuliaSourceId;
pub use parser_summary::julia_parser_summary_allows_safe_incremental_file_for_repository;
pub use parser_summary::julia_parser_summary_file_semantic_fingerprint_for_repository;
pub use parser_summary::set_linked_julia_parser_summary_base_url_for_tests;
pub use rerank_exchange::{
    fetch_julia_flight_score_rows_for_repository, fetch_plugin_arrow_score_rows_for_repository,
};
pub use transport::{
    JULIA_ARROW_RESPONSE_SCHEMA_VERSION, build_julia_flight_transport_client,
    process_julia_flight_batches, process_julia_flight_batches_for_repository,
};
pub use wendao_graph_evidence::{
    WENDAO_GRAPH_EVIDENCE_REQUEST_TABLE_CONTRACTS, WENDAO_GRAPH_EVIDENCE_REQUEST_TABLE_NAMES,
    WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_CONTRACTS, WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_NAMES,
    WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION, WENDAO_GRAPH_LINK_EVIDENCE_ROUTE,
    WendaoGraphEvidenceColumnContract, WendaoGraphEvidenceColumnType,
    WendaoGraphEvidenceTableContract, WendaoGraphEvidenceTableKind,
    is_wendao_graph_link_evidence_route, validate_wendao_graph_evidence_request_schema,
    validate_wendao_graph_evidence_response_schema, wendao_graph_evidence_request_table_contract,
    wendao_graph_evidence_response_table_contract, wendao_graph_evidence_table_schema,
    wendao_graph_link_evidence_route,
};
