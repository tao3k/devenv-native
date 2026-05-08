//! Integration fixtures for Julia-backed WendaoSearch examples and gateway artifacts.

mod gateway_artifact;
mod official_examples;
mod planned_search;
mod service_runtime;
mod wendaograph;
mod wendaograph_gnn;

pub use gateway_artifact::{
    julia_gateway_artifact_base_url, julia_gateway_artifact_expected_json_fragments,
    julia_gateway_artifact_expected_toml_fragments, julia_gateway_artifact_path,
    julia_gateway_artifact_rpc_params_fixture, julia_gateway_artifact_runtime_config_toml,
    julia_gateway_artifact_schema_version, julia_gateway_artifact_selected_transport,
    julia_ui_artifact_payload_fixture,
};
pub use official_examples::{
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
pub use planned_search::{
    julia_planned_search_openai_runtime_config_toml,
    julia_planned_search_vector_store_runtime_config_toml,
};
pub use service_runtime::JuliaExampleServiceGuard;
pub use wendaograph::{
    WendaoGraphLinkGraphFullStructuralHostProbeReport, WendaoGraphLinkGraphHostProbeReport,
    WendaoGraphPageIndexHostProbeReport, WendaoGraphPageIndexPlannerActionHostProbeReport,
    probe_wendaograph_link_graph_full_structural_host_request,
    probe_wendaograph_link_graph_host_request, probe_wendaograph_page_index_host_request,
    probe_wendaograph_page_index_host_request_with_fixture,
    probe_wendaograph_page_index_planner_action_host_request,
    probe_wendaograph_page_index_planner_action_host_request_with_fixture,
};
pub use wendaograph_gnn::{
    WendaoGraphGnnBackendLoadDiagnostics, WendaoGraphGnnHostProbeReport,
    probe_wendaograph_gnn_host_request,
};
