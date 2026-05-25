use super::{
    expected_wendaosearch_modelica_transport_contract, wendaocodeparser_config,
    wendaocodeparser_script, wendaosearch_parser_summary_contract,
};

#[test]
fn wendaosearch_parser_summary_contract_matches_rust_transport_constants() {
    let contract = wendaosearch_parser_summary_contract();
    let expected_transport = expected_wendaosearch_modelica_transport_contract();

    assert_eq!(contract.contract_version, 1);
    assert_eq!(
        contract.script_path(),
        wendaocodeparser_script("run_service.jl")
    );
    assert_eq!(
        contract.config_path(),
        wendaocodeparser_config("parser_summary.toml")
    );
    assert_eq!(contract.base_url(), "http://127.0.0.1:41081");
    assert_eq!(
        contract.service.default_code_parser_route_names,
        vec![
            "julia_file_summary".to_string(),
            "julia_root_summary".to_string(),
            "modelica_file_summary".to_string(),
            "modelica_ast_query".to_string(),
        ]
    );

    assert_eq!(contract.modelica_transport, expected_transport);
}
