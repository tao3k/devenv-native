use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepositoryPluginConfig};

use super::{
    JULIA_FILE_SUMMARY_ROUTE, JULIA_PARSER_SUMMARY_SCHEMA_VERSION, ParserSummaryRouteKind,
    build_julia_parser_summary_flight_transport_client,
    build_parser_summary_flight_transport_binding,
    clear_julia_parser_summary_transport_cache_for_tests,
    julia_parser_summary_transport_cache_len_for_tests,
    julia_parser_summary_transport_slot_id_for_tests,
    parser_summary_transport_error_requires_client_refresh,
};

#[test]
#[serial_test::serial(julia_parser_summary_transport)]
fn build_parser_summary_client_uses_default_discovery_for_plain_plugin_id() {
    clear_julia_parser_summary_transport_cache_for_tests();
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Id("julia-code-parser".to_string())],
        ..RegisteredRepository::default()
    };

    let client = build_julia_parser_summary_flight_transport_client(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("plain Julia plugin id should use default discovery: {error}"));

    assert!(!client.flight_base_url().trim().is_empty());
    assert_eq!(client.flight_route(), JULIA_FILE_SUMMARY_ROUTE);
    assert_eq!(
        client.selection().selected_transport,
        xiuxian_wendao_core::transport::PluginTransportKind::ArrowFlight,
    );
}

#[test]
#[serial_test::serial(julia_parser_summary_transport)]
fn build_parser_summary_client_reads_nested_options() {
    clear_julia_parser_summary_transport_cache_for_tests();
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "parser_summary_transport": {
                    "base_url": "http://127.0.0.1:9107",
                    "file_summary": {
                        "health_route": "/ready",
                        "timeout_secs": 21,
                        "max_in_flight_requests": 6
                    }
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let client = build_julia_parser_summary_flight_transport_client(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("parser-summary config should parse: {error}"));

    assert_eq!(client.flight_base_url(), "http://127.0.0.1:9107");
    assert_eq!(client.flight_route(), JULIA_FILE_SUMMARY_ROUTE);
    assert_eq!(
        client.selection().selected_transport,
        xiuxian_wendao_core::transport::PluginTransportKind::ArrowFlight,
    );
    let binding = build_parser_summary_flight_transport_binding(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("parser-summary binding should parse: {error}"));
    assert_eq!(binding.endpoint.max_in_flight_requests, Some(6));
    let _ = JULIA_PARSER_SUMMARY_SCHEMA_VERSION;
}

#[test]
#[serial_test::serial(julia_parser_summary_transport)]
fn build_parser_summary_client_rejects_disabled_transport() {
    clear_julia_parser_summary_transport_cache_for_tests();
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "parser_summary_transport": {
                    "enabled": false,
                    "base_url": "http://127.0.0.1:9107"
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let Err(error) = build_julia_parser_summary_flight_transport_client(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    ) else {
        panic!("disabled parser-summary transport must fail");
    };
    assert!(
        error
            .to_string()
            .contains("requires an enabled Julia parser-summary Flight transport client"),
        "unexpected error: {error}",
    );
}

#[test]
#[serial_test::serial(julia_parser_summary_transport)]
fn build_parser_summary_client_rejects_invalid_field_types() {
    clear_julia_parser_summary_transport_cache_for_tests();
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "parser_summary_transport": {
                    "root_summary": {
                        "timeout_secs": "fast"
                    }
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let error = build_julia_parser_summary_flight_transport_client(
        &repository,
        ParserSummaryRouteKind::RootSummary,
    )
    .err()
    .unwrap_or_else(|| panic!("invalid timeout type must fail"));
    assert!(
        error
            .to_string()
            .contains("Julia plugin field `timeout_secs` must be an unsigned integer"),
        "unexpected error: {error}",
    );
}

#[test]
#[serial_test::serial(julia_parser_summary_transport)]
fn build_parser_summary_client_reuses_cached_client_for_identical_transport() {
    clear_julia_parser_summary_transport_cache_for_tests();
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Id("julia-code-parser".to_string())],
        ..RegisteredRepository::default()
    };

    let first = build_julia_parser_summary_flight_transport_client(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("first Julia parser-summary client should build: {error}"));
    let first_slot = julia_parser_summary_transport_slot_id_for_tests(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("first Julia parser-summary slot should exist: {error}"));

    let second = build_julia_parser_summary_flight_transport_client(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("second Julia parser-summary client should build: {error}"));
    let second_slot = julia_parser_summary_transport_slot_id_for_tests(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("second Julia parser-summary slot should exist: {error}"));

    assert_eq!(first.flight_base_url(), second.flight_base_url());
    assert_eq!(first.flight_route(), second.flight_route());
    assert_eq!(first_slot, second_slot);
    assert_eq!(julia_parser_summary_transport_cache_len_for_tests(), 1);
}

#[test]
#[serial_test::serial(julia_parser_summary_transport)]
fn build_parser_summary_client_separates_cached_clients_by_route() {
    clear_julia_parser_summary_transport_cache_for_tests();
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Id("julia-code-parser".to_string())],
        ..RegisteredRepository::default()
    };

    build_julia_parser_summary_flight_transport_client(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("file-summary client should build: {error}"));
    let file_slot = julia_parser_summary_transport_slot_id_for_tests(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("file-summary slot should exist: {error}"));

    build_julia_parser_summary_flight_transport_client(
        &repository,
        ParserSummaryRouteKind::RootSummary,
    )
    .unwrap_or_else(|error| panic!("root-summary client should build: {error}"));
    let root_slot = julia_parser_summary_transport_slot_id_for_tests(
        &repository,
        ParserSummaryRouteKind::RootSummary,
    )
    .unwrap_or_else(|error| panic!("root-summary slot should exist: {error}"));

    assert_ne!(file_slot, root_slot);
    assert_eq!(julia_parser_summary_transport_cache_len_for_tests(), 2);
}

#[test]
#[serial_test::serial(julia_parser_summary_transport)]
fn build_parser_summary_client_separates_cached_clients_by_in_flight_budget() {
    clear_julia_parser_summary_transport_cache_for_tests();
    let repository_budget_three = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "parser_summary_transport": {
                    "base_url": "http://127.0.0.1:9107",
                    "max_in_flight_requests": 3
                }
            }),
        }],
        ..RegisteredRepository::default()
    };
    let repository_budget_five = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "parser_summary_transport": {
                    "base_url": "http://127.0.0.1:9107",
                    "max_in_flight_requests": 5
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    build_julia_parser_summary_flight_transport_client(
        &repository_budget_three,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("budget-three client should build: {error}"));
    let slot_three = julia_parser_summary_transport_slot_id_for_tests(
        &repository_budget_three,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("budget-three slot should exist: {error}"));

    build_julia_parser_summary_flight_transport_client(
        &repository_budget_five,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("budget-five client should build: {error}"));
    let slot_five = julia_parser_summary_transport_slot_id_for_tests(
        &repository_budget_five,
        ParserSummaryRouteKind::FileSummary,
    )
    .unwrap_or_else(|error| panic!("budget-five slot should exist: {error}"));

    assert_ne!(slot_three, slot_five);
    assert_eq!(julia_parser_summary_transport_cache_len_for_tests(), 2);
}

#[test]
fn parser_summary_transport_refreshes_dispatch_gone_errors() {
    let dispatch_gone_error =
        xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError::AnalysisFailed {
        message: "Julia parser-summary Flight request failed: Tonic error: code: 'Unknown error', message: \"transport error\", source: tonic::transport::Error(Transport, hyper::Error(User(DispatchGone), \"runtime dropped the dispatch task\"))".to_string(),
    };
    let transport_closed_error =
        xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError::AnalysisFailed {
            message: "Julia parser-summary Flight request failed: Tonic error: code: 'Unknown error', message: \"transport error\", source: tonic::transport::Error(Transport, Closed)"
                .to_string(),
        };

    assert!(parser_summary_transport_error_requires_client_refresh(
        &dispatch_gone_error
    ));
    assert!(parser_summary_transport_error_requires_client_refresh(
        &transport_closed_error
    ));
}
