//! Julia core contracts and Wendao integration adapters.

mod arrow_metadata;
/// Compatibility surfaces exported by the Julia core crate.
pub mod compatibility;
/// Bounded integration-test helpers for Julia-owned managed services.
pub mod integration_support;
/// Runtime-level memory-family thin compat surfaces for Julia compute.
pub mod memory;
mod modelica_plugin;
mod plugin;
/// Read-only projections from Julia-owned contracts into polyglot contracts.
pub mod polyglot;
mod semantic;
/// Command-line adapters for Julia-owned Wendao bridges.
pub mod wendaograph_search_strategy_flow_cli;

pub use semantic::{
    JuliaContractEnabled, JuliaContractId, JuliaContractKind, JuliaContractMode, JuliaContractPath,
    JuliaContractReason, JuliaContractRoute, JuliaContractSchemaVersion, JuliaContractSecondsU64,
    JuliaContractState, JuliaContractTimestampMsI64, JuliaContractTransport, JuliaContractUrl,
};

pub(crate) use modelica_plugin::fetch_modelica_parser_file_summary_blocking_for_repository;
#[cfg(test)]
pub(crate) use plugin::test_support as julia_plugin_test_support;

pub use modelica_plugin::{
    ModelicaRepoIntelligencePlugin, ModelicaSourceId,
    clear_modelica_parser_summary_transport_cache_for_tests,
    fetch_modelica_ast_query_analysis_blocking_for_repository,
    modelica_package_incremental_semantic_fingerprint_for_repository,
    modelica_parser_summary_allows_safe_incremental_file_for_repository,
    modelica_parser_summary_allows_safe_package_incremental_file_for_repository,
    modelica_parser_summary_allows_safe_root_package_incremental_file_for_repository,
    modelica_parser_summary_file_semantic_fingerprint_for_repository,
    modelica_parser_summary_root_package_name_matches_repository_context,
    modelica_root_package_incremental_semantic_fingerprint_for_repository, register_modelica_into,
    set_linked_modelica_parser_summary_base_url_for_tests,
};
pub use plugin::{
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
    GraphStructuralCandidateSubgraph, GraphStructuralCandidateSubgraphInput,
    GraphStructuralFilterConstraint, GraphStructuralFilterRequestRow,
    GraphStructuralFilterScoreRow, GraphStructuralGenericTopologyCandidateInput,
    GraphStructuralGenericTopologyCandidateInputs,
    GraphStructuralGenericTopologyCandidateMetadataInput,
    GraphStructuralGenericTopologyCandidateMetadataInputs,
    GraphStructuralGenericTopologyPairCollectionInput, GraphStructuralKeywordOverlapCandidateInput,
    GraphStructuralKeywordOverlapCandidateInputs,
    GraphStructuralKeywordOverlapCandidateMetadataInput,
    GraphStructuralKeywordOverlapCandidateMetadataInputs, GraphStructuralKeywordOverlapPairInputs,
    GraphStructuralKeywordOverlapPairRequestInput, GraphStructuralKeywordOverlapPairRequestInputs,
    GraphStructuralKeywordOverlapPairRerankInput, GraphStructuralKeywordOverlapPairRerankInputs,
    GraphStructuralKeywordOverlapPairRerankRowInput, GraphStructuralKeywordOverlapQueryInput,
    GraphStructuralKeywordOverlapQueryInputs, GraphStructuralKeywordOverlapRawCandidateInput,
    GraphStructuralKeywordOverlapRawCandidateInputs, GraphStructuralKeywordTagMatchFlags,
    GraphStructuralKeywordTagPairRerankRequestInput, GraphStructuralKeywordTagQueryContextInput,
    GraphStructuralKeywordTagQueryInput, GraphStructuralKeywordTagQueryInputs,
    GraphStructuralKeywordTagRerankSignalInput, GraphStructuralNodeMetadataInputs,
    GraphStructuralPairCandidateInputs, GraphStructuralPairFilterRequestInput,
    GraphStructuralPairRerankRequestInput, GraphStructuralQueryAnchor, GraphStructuralQueryContext,
    GraphStructuralQueryContextInput, GraphStructuralRawConnectedPairCandidateInput,
    GraphStructuralRawConnectedPairCollectionCandidateInputs,
    GraphStructuralRawConnectedPairCollectionRawTupleInput, GraphStructuralRawConnectedPairInputs,
    GraphStructuralRerankRequestRow, GraphStructuralRerankScoreRow, GraphStructuralRerankSignals,
    GraphStructuralRouteKind, GraphStructuralScoredPairCandidateInputs,
    GraphStructuralScoredPairCollectionCandidateInput, JULIA_ARROW_RESPONSE_SCHEMA_VERSION,
    JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION, JULIA_PLUGIN_CAPABILITY_MANIFEST_BASE_URL_COLUMN,
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
    JuliaRepoIntelligencePlugin, JuliaSourceId, WENDAO_GRAPH_EVIDENCE_REQUEST_TABLE_CONTRACTS,
    WENDAO_GRAPH_EVIDENCE_REQUEST_TABLE_NAMES, WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_CONTRACTS,
    WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_NAMES, WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION,
    WENDAO_GRAPH_LINK_EVIDENCE_ROUTE, WENDAO_GRAPH_PAGE_INDEX_REASONING_REQUEST_TABLE_CONTRACTS,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_REQUEST_TABLE_NAMES,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_RESPONSE_TABLE_CONTRACTS,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_RESPONSE_TABLE_NAMES, WendaoGraphEvidenceColumnContract,
    WendaoGraphEvidenceColumnType, WendaoGraphEvidenceTableContract, WendaoGraphEvidenceTableKind,
    build_graph_structural_filter_request_batch, build_graph_structural_filter_request_row,
    build_graph_structural_flight_transport_client,
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
    build_graph_structural_raw_connected_pair_inputs, build_graph_structural_rerank_request_batch,
    build_graph_structural_rerank_request_row, build_graph_structural_scored_pair_candidate_inputs,
    build_julia_capability_manifest_flight_transport_client, build_julia_flight_transport_client,
    build_julia_plugin_capability_manifest_request_batch,
    decode_graph_structural_filter_score_rows, decode_graph_structural_rerank_score_rows,
    decode_julia_plugin_capability_manifest_rows,
    fetch_graph_structural_filter_rows_for_repository,
    fetch_graph_structural_generic_topology_filter_rows_for_repository,
    fetch_graph_structural_generic_topology_filter_rows_for_repository_from_raw_connected_pair_collections,
    fetch_graph_structural_generic_topology_rerank_rows_for_repository,
    fetch_graph_structural_generic_topology_rerank_rows_for_repository_from_raw_connected_pair_collections,
    fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository,
    fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates,
    fetch_graph_structural_rerank_rows_for_repository,
    fetch_julia_flight_score_rows_for_repository,
    fetch_julia_plugin_capability_manifest_rows_for_repository,
    fetch_plugin_arrow_score_rows_for_repository, graph_structural_pair_candidate_id,
    graph_structural_route_kind, graph_structural_shared_tag_anchors, is_graph_structural_route,
    is_wendao_graph_link_evidence_route,
    julia_parser_summary_allows_safe_incremental_file_for_repository,
    julia_parser_summary_file_semantic_fingerprint_for_repository,
    process_graph_structural_flight_batches,
    process_graph_structural_flight_batches_for_repository,
    process_julia_capability_manifest_flight_batches,
    process_julia_capability_manifest_flight_batches_for_repository, process_julia_flight_batches,
    process_julia_flight_batches_for_repository, register_into,
    set_linked_julia_parser_summary_base_url_for_tests,
    validate_graph_structural_filter_request_batch,
    validate_graph_structural_filter_request_schema,
    validate_graph_structural_filter_response_batch,
    validate_graph_structural_filter_response_schema, validate_graph_structural_request_batches,
    validate_graph_structural_rerank_request_batch,
    validate_graph_structural_rerank_request_schema,
    validate_graph_structural_rerank_response_batch,
    validate_graph_structural_rerank_response_schema, validate_graph_structural_response_batches,
    validate_julia_plugin_capability_manifest_request_batches,
    validate_julia_plugin_capability_manifest_response_batches,
    validate_wendao_graph_evidence_request_schema, validate_wendao_graph_evidence_response_schema,
    validate_wendao_graph_page_index_reasoning_request_schema,
    validate_wendao_graph_page_index_reasoning_response_schema,
    wendao_graph_evidence_request_table_contract, wendao_graph_evidence_response_table_contract,
    wendao_graph_evidence_table_schema, wendao_graph_link_evidence_route,
    wendao_graph_page_index_reasoning_request_table_contract,
    wendao_graph_page_index_reasoning_response_table_contract,
    wendao_graph_page_index_reasoning_table_schema,
};

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = {
        rust_lang_project_harness::default_rust_harness_config().with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/lib.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_rationale("crate root owns the public package API for cargo-test verification"),
        )
        .with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/polyglot/",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_task_kinds([rust_lang_project_harness::RustVerificationTaskKind::Regression])
            .with_task_contract(
                rust_lang_project_harness::RustVerificationTaskKind::Regression,
                rust_lang_project_harness::RustVerificationTaskContract::new(
                    rust_lang_project_harness::RustVerificationPhase::AfterUnitTestsPass,
                    "Regression check must exercise the Julia polyglot readiness bridge",
                    [
                        rust_lang_project_harness::RustVerificationRequirement::new(
                            "command",
                            "cargo test -p xiuxian-julia-core --lib polyglot",
                        ),
                        rust_lang_project_harness::RustVerificationRequirement::new(
                            "target",
                            "lib unit tests mounted from tests/unit/polyglot/",
                        ),
                        rust_lang_project_harness::RustVerificationRequirement::new(
                            "coverage",
                            "profile refs, manifest refs, readiness evidence, admission, and snapshots",
                        ),
                    ],
                ),
            )
            .with_rationale(
                "Julia polyglot bridge owns readiness evidence projections for the orchestrator chain",
            ),
        )
    }
);
