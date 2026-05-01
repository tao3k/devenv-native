mod common;
mod gateway_artifact;
mod official_examples;
mod planned_search;

pub use common::JuliaExampleServiceGuard;
pub use gateway_artifact::{
    julia_gateway_artifact_base_url, julia_gateway_artifact_expected_json_fragments,
    julia_gateway_artifact_expected_toml_fragments, julia_gateway_artifact_path,
    julia_gateway_artifact_rpc_params_fixture, julia_gateway_artifact_runtime_config_toml,
    julia_gateway_artifact_schema_version, julia_gateway_artifact_selected_transport,
    julia_ui_artifact_payload_fixture,
};
pub use official_examples::{
    probe_wendaosearch_modelica_parser_summary_route_for_tests,
    spawn_wendaosearch_all_parser_summary_service, spawn_wendaosearch_demo_multi_route_service,
    spawn_wendaosearch_demo_structural_rerank_service,
    spawn_wendaosearch_julia_parser_summary_service,
    spawn_wendaosearch_julia_parser_summary_service_with_attempts,
    spawn_wendaosearch_modelica_parser_summary_service,
    spawn_wendaosearch_solver_demo_multi_route_service,
    spawn_wendaosearch_solver_demo_structural_rerank_service,
};
pub use planned_search::{
    julia_planned_search_openai_runtime_config_toml,
    julia_planned_search_vector_store_runtime_config_toml,
};
