//! Integration fixtures for Julia-backed `WendaoSearch` services and gateway artifacts.

mod gateway_artifact;
mod planned_search;
mod search_strategy_flow_candidates;
mod search_strategy_flow_evidence;
mod search_strategy_flow_flight;
mod service_runtime;
mod wendaograph;
mod wendaograph_gnn;
mod wendaosearch_services;

pub use gateway_artifact::{
    julia_gateway_artifact_base_url, julia_gateway_artifact_expected_json_fragments,
    julia_gateway_artifact_expected_toml_fragments, julia_gateway_artifact_path,
    julia_gateway_artifact_rpc_params_fixture, julia_gateway_artifact_runtime_config_toml,
    julia_gateway_artifact_schema_version, julia_gateway_artifact_selected_transport,
    julia_ui_artifact_payload_fixture,
};
pub use planned_search::{
    julia_planned_search_openai_runtime_config_toml,
    julia_planned_search_vector_store_runtime_config_toml,
};
pub(crate) use search_strategy_flow_evidence::search_strategy_flow_evidence_edge_kinds;
pub use search_strategy_flow_flight::{
    SearchStrategyFlowFlightMaterializationConfig, materialize_search_strategy_flow_routes,
};
pub use service_runtime::JuliaServiceGuard;
pub(crate) use service_runtime::{
    repo_root, reserve_service_port, wait_for_service_ready_with_attempts,
    wendaosearch_julia_project, wendaosearch_script,
};
pub use wendaograph::ontology_read_model::{
    WENDAO_GRAPH_ONTOLOGY_EXTENSION_DOMAIN_PREFIX_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_EXTENSION_PROOF_REQUEST_TABLES,
    WENDAO_GRAPH_ONTOLOGY_EXTENSION_PROOF_RESPONSE_TABLE,
    WENDAO_GRAPH_ONTOLOGY_PARENT_LINK_TYPES_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_PARENT_LINK_TYPES_TABLE,
    WENDAO_GRAPH_ONTOLOGY_PARENT_OBJECT_TYPES_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_PARENT_OBJECT_TYPES_TABLE, WENDAO_GRAPH_ONTOLOGY_RDF_NAMESPACE_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ARROW_IPC_MIME,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_CAPABILITY_ID,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_FLIGHT_DESCRIPTOR_PATH,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_METHOD,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_PROFILE_ID,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_PROVIDER_ID,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_BUNDLE_TABLE,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_TABLES,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_RESPONSE_TABLE,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ROUTE,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SERVICE,
    WENDAO_GRAPH_ONTOLOGY_SEMANTIC_OBJECTS_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_SEMANTIC_PROJECTION_STATE_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_SEMANTIC_RELATIONS_PAYLOAD_COLUMN,
    WendaoGraphOntologyExtensionProofArrowRequest, WendaoGraphOntologyExtensionProofRequestBatches,
    WendaoGraphOntologyReadModelQualityArrowRequest,
    WendaoGraphOntologyReadModelQualityFlightBindingOptions,
    WendaoGraphOntologyReadModelQualityRequestBatches,
    WendaoGraphOntologyReadModelQualityRoundtrip,
    WendaoGraphOntologyReadModelQualityRoundtripError,
    build_wendaograph_ontology_extension_proof_arrow_request,
    build_wendaograph_ontology_extension_proof_flight_request_batch,
    build_wendaograph_ontology_read_model_quality_arrow_request,
    build_wendaograph_ontology_read_model_quality_flight_binding,
    build_wendaograph_ontology_read_model_quality_flight_descriptor,
    build_wendaograph_ontology_read_model_quality_flight_request_batch,
    build_wendaograph_ontology_read_model_quality_orchestrator_schedule_plan,
    build_wendaograph_ontology_read_model_quality_request_batches_from_dataset_ontology_envelope,
    build_wendaograph_ontology_read_model_quality_request_batches_from_rdf_source_artifacts,
    build_wendaograph_ontology_read_model_quality_request_batches_from_semantic_preview_artifacts,
    roundtrip_wendaograph_ontology_extension_proof_with_binding,
    roundtrip_wendaograph_ontology_read_model_quality_with_binding,
    wendaograph_ontology_read_model_quality_provider_selector,
    wendaograph_ontology_read_model_quality_route_profile_ref,
};
pub use wendaograph::{
    SearchStrategyFlowPersistentBatchHost, SearchStrategyFlowPersistentHostStabilizationLimits,
    SearchStrategyFlowPersistentHostStabilizationReason,
    SearchStrategyFlowPersistentHostStabilizationReport,
    SearchStrategyFlowPersistentHostWarmPathStats, SearchStrategyFlowProbeAction,
};
pub use wendaograph::{
    WendaoGraphLinkGraphFullStructuralHostProbeReport, WendaoGraphLinkGraphHostProbeReport,
    WendaoGraphPageIndexHostProbeReport, WendaoGraphPageIndexPlannerActionHostProbeReport,
    enrich_wendaograph_search_strategy_flow_retrieval_routes,
    enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization,
    parse_search_strategy_flow_probe_action,
    probe_wendaograph_link_graph_full_structural_host_request,
    probe_wendaograph_link_graph_host_request, probe_wendaograph_page_index_host_request,
    probe_wendaograph_page_index_host_request_with_fixture,
    probe_wendaograph_page_index_planner_action_host_request,
    probe_wendaograph_page_index_planner_action_host_request_with_fixture,
    run_wendaograph_search_strategy_flow_json,
    run_wendaograph_search_strategy_flow_json_batch_with_candidate_batches,
    run_wendaograph_search_strategy_flow_json_with_flight_materialization,
    run_wendaograph_search_strategy_flow_json_with_flight_materialization_and_branch_judgements,
    search_strategy_flow_probe_action_route,
};
pub use wendaograph_gnn::{
    WendaoGraphGnnBackendLoadDiagnostics, WendaoGraphGnnHostProbeReport,
    probe_wendaograph_gnn_host_request,
};
pub use wendaosearch_services::{
    WendaoSearchGraphStructuralPrewarmReport, WendaoSearchGraphStructuralStabilizationLimits,
    WendaoSearchGraphStructuralStabilizationReason, WendaoSearchGraphStructuralStabilizationReport,
    WendaoSearchGraphStructuralWarmPathStats,
    prewarm_wendaosearch_solver_demo_graph_structural_routes,
    probe_wendaosearch_modelica_parser_summary_route_for_tests,
    spawn_wendaosearch_all_parser_summary_service, spawn_wendaosearch_demo_multi_route_service,
    spawn_wendaosearch_demo_structural_rerank_service,
    spawn_wendaosearch_julia_parser_summary_service,
    spawn_wendaosearch_julia_parser_summary_service_with_attempts,
    spawn_wendaosearch_modelica_parser_summary_service,
    spawn_wendaosearch_solver_demo_multi_route_service,
    spawn_wendaosearch_solver_demo_structural_rerank_service,
    stabilize_wendaosearch_solver_demo_graph_structural_routes,
};
