use xiuxian_wendao_core::repo_intelligence::{
    RegisteredRepository, RepoIntelligenceError, RepositoryPluginConfig,
};

use super::{
    MODELICA_FILE_SUMMARY_ROUTE, ParserSummaryRouteKind,
    build_modelica_parser_summary_flight_transport_client,
    build_parser_summary_flight_transport_binding,
    clear_modelica_parser_summary_transport_cache_for_tests,
    modelica_parser_summary_transport_cache_len_for_tests,
    modelica_parser_summary_transport_slot_id_for_tests,
    parser_summary_transport_error_requires_client_refresh,
};

fn parser_summary_repository() -> RegisteredRepository {
    RegisteredRepository {
        id: "repo-modelica".to_string(),
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
        ..RegisteredRepository::default()
    }
}

#[test]
#[serial_test::serial(modelica_parser_summary_transport)]
fn build_modelica_parser_summary_client_reads_nested_options() {
    clear_modelica_parser_summary_transport_cache_for_tests();
    let repository = RegisteredRepository {
        id: "repo-modelica".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "modelica".to_string(),
            options: serde_json::json!({
                "parser_summary_transport": {
                    "base_url": "http://127.0.0.1:9207",
                    "file_summary": {
                        "health_route": "/ready",
                        "timeout_secs": 21,
                        "max_in_flight_requests": 5
                    }
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let client = build_modelica_parser_summary_flight_transport_client(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("Modelica parser-summary config should parse: {error}"));

    assert_eq!(client.flight_base_url(), "http://127.0.0.1:9207");
    assert_eq!(client.flight_route(), MODELICA_FILE_SUMMARY_ROUTE);
    let binding = build_parser_summary_flight_transport_binding(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("Modelica parser-summary binding should parse: {error}"));
    assert_eq!(binding.endpoint.max_in_flight_requests, Some(5));
}

#[test]
#[serial_test::serial(modelica_parser_summary_transport)]
fn build_modelica_parser_summary_client_uses_default_discovery_for_plain_plugin_id() {
    clear_modelica_parser_summary_transport_cache_for_tests();
    let repository = parser_summary_repository();

    let client = build_modelica_parser_summary_flight_transport_client(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| {
        panic!("plain Modelica plugin id should use default discovery: {error}")
    });

    assert!(!client.flight_base_url().trim().is_empty());
    assert_eq!(client.flight_route(), MODELICA_FILE_SUMMARY_ROUTE);
    assert_eq!(
        client.selection().selected_transport,
        xiuxian_wendao_core::transport::PluginTransportKind::ArrowFlight,
    );
    let binding = build_parser_summary_flight_transport_binding(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("Modelica parser-summary binding should parse: {error}"));
    assert_eq!(binding.endpoint.max_in_flight_requests, Some(1));
}

#[test]
#[serial_test::serial(modelica_parser_summary_transport)]
fn build_modelica_parser_summary_client_reuses_cached_transport_slot_for_same_binding() {
    clear_modelica_parser_summary_transport_cache_for_tests();
    let repository = parser_summary_repository();
    let baseline = modelica_parser_summary_transport_cache_len_for_tests();

    let _client_a = build_modelica_parser_summary_flight_transport_client(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| {
        panic!("first Modelica parser-summary client build should succeed: {error}")
    });
    let slot_a = modelica_parser_summary_transport_slot_id_for_tests(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("expected cached slot after first build: {error}"));

    let _client_b = build_modelica_parser_summary_flight_transport_client(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| {
        panic!("second Modelica parser-summary client build should succeed: {error}")
    });
    let slot_b = modelica_parser_summary_transport_slot_id_for_tests(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("expected cached slot after second build: {error}"));

    assert_eq!(slot_a, slot_b);
    assert_eq!(
        modelica_parser_summary_transport_cache_len_for_tests(),
        baseline + 1
    );
}

#[test]
#[serial_test::serial(modelica_parser_summary_transport)]
fn clear_modelica_parser_summary_transport_cache_drops_cached_slot() {
    let repository = parser_summary_repository();
    clear_modelica_parser_summary_transport_cache_for_tests();

    let _client_a = build_modelica_parser_summary_flight_transport_client(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| {
        panic!("first Modelica parser-summary client build should succeed: {error}")
    });
    let slot_a = modelica_parser_summary_transport_slot_id_for_tests(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("expected cached slot after first build: {error}"));

    clear_modelica_parser_summary_transport_cache_for_tests();
    assert_eq!(modelica_parser_summary_transport_cache_len_for_tests(), 0);

    let _client_b = build_modelica_parser_summary_flight_transport_client(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| {
        panic!("second Modelica parser-summary client build should succeed: {error}")
    });
    let slot_b = modelica_parser_summary_transport_slot_id_for_tests(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("expected cached slot after cache clear: {error}"));

    assert_ne!(slot_a, slot_b);
    assert_eq!(modelica_parser_summary_transport_cache_len_for_tests(), 1);
}

#[test]
#[serial_test::serial(modelica_parser_summary_transport)]
fn build_modelica_parser_summary_client_separates_cached_clients_by_in_flight_budget() {
    clear_modelica_parser_summary_transport_cache_for_tests();
    let repository_budget_three = RegisteredRepository {
        id: "repo-modelica".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "modelica".to_string(),
            options: serde_json::json!({
                "parser_summary_transport": {
                    "base_url": "http://127.0.0.1:9207",
                    "max_in_flight_requests": 3
                }
            }),
        }],
        ..RegisteredRepository::default()
    };
    let repository_budget_five = RegisteredRepository {
        id: "repo-modelica".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "modelica".to_string(),
            options: serde_json::json!({
                "parser_summary_transport": {
                    "base_url": "http://127.0.0.1:9207",
                    "max_in_flight_requests": 5
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let _client_three = build_modelica_parser_summary_flight_transport_client(
        &repository_budget_three,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("budget-three Modelica client should build: {error}"));
    let slot_three = modelica_parser_summary_transport_slot_id_for_tests(
        &repository_budget_three,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("budget-three Modelica slot should exist: {error}"));

    let _client_five = build_modelica_parser_summary_flight_transport_client(
        &repository_budget_five,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("budget-five Modelica client should build: {error}"));
    let slot_five = modelica_parser_summary_transport_slot_id_for_tests(
        &repository_budget_five,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("budget-five Modelica slot should exist: {error}"));

    assert_ne!(slot_three, slot_five);
    assert!(
        modelica_parser_summary_transport_cache_len_for_tests() >= 2,
        "cache should retain distinct entries for both in-flight budgets"
    );
}

#[test]
#[serial_test::serial(modelica_parser_summary_transport)]
fn parser_summary_transport_refresh_detects_stale_service_channel_errors() {
    let stale_transport_error = RepoIntelligenceError::AnalysisFailed {
        message: "Modelica parser-summary Flight request for route `/wendao/code-parser/modelica/ast-query` failed: Arrow Flight request failed: Tonic error: code: 'Unknown error', message: \"Service was not ready: transport error\"".to_string(),
    };
    let ordinary_analysis_error = RepoIntelligenceError::AnalysisFailed {
        message: "Modelica parser-summary Flight request failed: some parser error".to_string(),
    };
    let connection_reset_error = RepoIntelligenceError::AnalysisFailed {
        message: "Modelica parser-summary Flight request for route `/wendao/code-parser/modelica/file-summary` failed: Arrow Flight request failed: Tonic error: code: 'Unknown error', message: \"transport error\", source: tonic::transport::Error(Transport, hyper::Error(Io, Kind(ConnectionReset)))".to_string(),
    };
    let broken_pipe_error = RepoIntelligenceError::AnalysisFailed {
        message: "Modelica parser-summary Flight request for route `/wendao/code-parser/modelica/file-summary` failed: Arrow Flight request failed: Tonic error: code: 'Unknown error', message: \"transport error\", source: tonic::transport::Error(Transport, hyper::Error(Io, Custom { kind: BrokenPipe, error: \"stream closed because of a broken pipe\" }))".to_string(),
    };
    let transport_closed_error = RepoIntelligenceError::AnalysisFailed {
        message: "Modelica parser-summary Flight request for route `/wendao/code-parser/modelica/ast-query` failed: Arrow Flight request failed: Tonic error: code: 'Unknown error', message: \"transport error\", source: tonic::transport::Error(Transport, Closed)".to_string(),
    };

    assert!(parser_summary_transport_error_requires_client_refresh(
        &stale_transport_error
    ));
    assert!(parser_summary_transport_error_requires_client_refresh(
        &connection_reset_error
    ));
    assert!(parser_summary_transport_error_requires_client_refresh(
        &broken_pipe_error
    ));
    assert!(parser_summary_transport_error_requires_client_refresh(
        &transport_closed_error
    ));
    assert!(!parser_summary_transport_error_requires_client_refresh(
        &ordinary_analysis_error
    ));
}
