//! Host-process probes for local `WendaoGraph.jl` contracts.

#[path = "../wendaograph_batch_replay.rs"]
mod batch_replay;
mod constants;
mod host;
pub mod ontology_read_model;
#[path = "../wendaograph_persistent_host_report.rs"]
mod persistent_host_report;
#[path = "../wendaograph_probes.rs"]
mod probes;
#[path = "../wendaograph_scripts.rs"]
mod scripts;
mod search_strategy_routes;

#[cfg(test)]
pub(crate) use crate::integration_support::search_strategy_flow_candidates::{
    SearchStrategyFlowCandidateInputBatch,
    search_strategy_flow_registry_authority_candidate_input_batch,
};
#[cfg(test)]
pub(crate) use crate::integration_support::search_strategy_flow_flight::{
    SearchStrategyFlowFlightMaterializationConfig,
    search_strategy_flow_candidate_input_batch_from_repo_search,
};
pub use batch_replay::{
    SearchStrategyFlowPersistentBatchHost,
    run_wendaograph_search_strategy_flow_json_batch_with_candidate_batches,
};
pub(crate) use constants::{
    LINK_GRAPH_HOST_PROBE_PREFIX, PAGE_INDEX_HOST_PROBE_PREFIX,
    WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_WARM_SAMPLES_ENV,
    WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_FANOUT_ENV, WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_NODES_ENV,
    WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_SEMANTIC_NEIGHBORS_ENV,
    WENDAO_GRAPH_PAGE_INDEX_HOST_FIXTURE_ENV, WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS_ENV,
    WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_WARM_SAMPLES_ENV, WENDAOGRAPH_JULIA_PROJECT_ENV,
    WENDAOGRAPH_PACKAGE_DIR_ENV,
};
pub(crate) use host::validate_search_strategy_flow_intent;
pub use host::{
    SearchStrategyFlowProbeAction, enrich_wendaograph_search_strategy_flow_retrieval_routes,
    enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization,
    parse_search_strategy_flow_probe_action, run_wendaograph_search_strategy_flow_json,
    run_wendaograph_search_strategy_flow_json_with_flight_materialization,
    run_wendaograph_search_strategy_flow_json_with_flight_materialization_and_branch_judgements,
    search_strategy_flow_probe_action_route,
};
#[cfg(test)]
pub(crate) use host::{
    configured_wendaograph_search_strategy_flow_markdown_replay_families,
    configured_wendaograph_search_strategy_flow_markdown_replay_families_with_limit,
    run_wendaograph_search_strategy_flow_json_with_candidate_batch,
    run_wendaograph_search_strategy_flow_json_with_candidate_batch_and_branch_judgements,
};
#[cfg(test)]
pub(crate) use ontology_read_model::{
    WENDAO_GRAPH_ONTOLOGY_EXTENSION_DOMAIN_PREFIX_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_EXTENSION_PROOF_REQUEST_TABLES,
    WENDAO_GRAPH_ONTOLOGY_EXTENSION_PROOF_RESPONSE_TABLE,
    WENDAO_GRAPH_ONTOLOGY_PARENT_LINK_TYPES_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_PARENT_OBJECT_TYPES_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_RDF_NAMESPACE_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ARROW_IPC_MIME,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_CAPABILITY_ID,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_FLIGHT_DESCRIPTOR_PATH,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_METHOD,
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
    build_wendaograph_ontology_extension_proof_arrow_request,
    build_wendaograph_ontology_extension_proof_flight_request_batch,
    build_wendaograph_ontology_read_model_quality_arrow_request,
    build_wendaograph_ontology_read_model_quality_flight_binding,
    build_wendaograph_ontology_read_model_quality_flight_descriptor,
    build_wendaograph_ontology_read_model_quality_flight_request_batch,
    build_wendaograph_ontology_read_model_quality_request_batches_from_dataset_ontology_envelope,
    roundtrip_wendaograph_ontology_read_model_quality_with_binding,
    wendaograph_ontology_read_model_quality_provider_selector,
};
pub use persistent_host_report::{
    SearchStrategyFlowPersistentHostStabilizationLimits,
    SearchStrategyFlowPersistentHostStabilizationReason,
    SearchStrategyFlowPersistentHostStabilizationReport,
    SearchStrategyFlowPersistentHostWarmPathStats,
};
pub use probes::{
    WendaoGraphLinkGraphFullStructuralHostProbeReport, WendaoGraphLinkGraphHostProbeReport,
    WendaoGraphPageIndexHostProbeReport, WendaoGraphPageIndexPlannerActionHostProbeReport,
    probe_wendaograph_link_graph_full_structural_host_request,
    probe_wendaograph_link_graph_host_request, probe_wendaograph_page_index_host_request,
    probe_wendaograph_page_index_host_request_with_fixture,
    probe_wendaograph_page_index_planner_action_host_request,
    probe_wendaograph_page_index_planner_action_host_request_with_fixture,
};
#[cfg(test)]
pub(crate) use probes::{
    parse_link_graph_full_structural_probe_report_line, parse_link_graph_probe_report_line,
    parse_page_index_planner_action_probe_report_line, parse_page_index_probe_report_line,
};
pub(crate) use probes::{resolve_existing_path, wendaograph_julia_project};

#[cfg(test)]
#[path = "../../../tests/unit/integration_support/wendaograph/mod.rs"]
mod tests;
