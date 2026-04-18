#[test]
fn build_graph_structural_flight_transport_client_returns_none_without_config() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Id("julia".to_string())],
        ..RegisteredRepository::default()
    };

    let client = build_graph_structural_flight_transport_client(
        &repository,
        GraphStructuralRouteKind::StructuralRerank,
    )
    .unwrap_or_else(|error| panic!("missing graph-structural config should be ignored: {error}"));
    assert!(client.is_none());
}

#[test]
fn build_graph_structural_flight_transport_client_reads_common_options() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia".to_string(),
            options: serde_json::json!({
                "graph_structural_transport": {
                    "base_url": "http://127.0.0.1:9101",
                    "health_route": "/ready",
                    "timeout_secs": 25,
                    "max_in_flight_requests": 4
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let client = build_graph_structural_flight_transport_client(
        &repository,
        GraphStructuralRouteKind::StructuralRerank,
    )
    .unwrap_or_else(|error| panic!("graph-structural config should parse: {error}"))
    .unwrap_or_else(|| panic!("graph-structural client should exist"));

    assert_eq!(client.flight_base_url(), "http://127.0.0.1:9101");
    assert_eq!(client.flight_route(), GRAPH_STRUCTURAL_RERANK_ROUTE);
    assert_eq!(
        client.selection().selected_transport,
        PluginTransportKind::ArrowFlight
    );
    let binding = build_graph_structural_flight_transport_binding(
        &repository,
        GraphStructuralRouteKind::StructuralRerank,
    )
    .unwrap_or_else(|error| panic!("graph-structural binding should parse: {error}"))
    .unwrap_or_else(|| panic!("graph-structural binding should exist"));
    assert_eq!(binding.endpoint.max_in_flight_requests, Some(4));
}

#[test]
fn build_graph_structural_flight_transport_client_reads_route_specific_overrides() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia".to_string(),
            options: serde_json::json!({
                "graph_structural_transport": {
                    "base_url": "http://127.0.0.1:9101",
                    "structural_rerank": {
                        "route": "graph/structural/rerank",
                        "schema_version": "v0-custom",
                        "timeout_secs": 30
                    },
                    "constraint_filter": {
                        "route": "/graph/structural/filter",
                        "timeout_secs": 12
                    }
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let rerank_client = build_graph_structural_flight_transport_client(
        &repository,
        GraphStructuralRouteKind::StructuralRerank,
    )
    .unwrap_or_else(|error| panic!("rerank config should parse: {error}"))
    .unwrap_or_else(|| panic!("rerank client should exist"));
    let filter_client = build_graph_structural_flight_transport_client(
        &repository,
        GraphStructuralRouteKind::ConstraintFilter,
    )
    .unwrap_or_else(|error| panic!("filter config should parse: {error}"))
    .unwrap_or_else(|| panic!("filter client should exist"));

    assert_eq!(rerank_client.flight_route(), GRAPH_STRUCTURAL_RERANK_ROUTE);
    assert_eq!(filter_client.flight_route(), GRAPH_STRUCTURAL_FILTER_ROUTE);
}

#[test]
fn build_graph_structural_flight_transport_client_honors_enabled_false() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia".to_string(),
            options: serde_json::json!({
                "graph_structural_transport": {
                    "base_url": "http://127.0.0.1:9101",
                    "constraint_filter": {
                        "enabled": false
                    }
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let client = build_graph_structural_flight_transport_client(
        &repository,
        GraphStructuralRouteKind::ConstraintFilter,
    )
    .unwrap_or_else(|error| panic!("disabled route-specific config should parse: {error}"));
    assert!(client.is_none());
}

#[test]
fn build_graph_structural_flight_transport_client_rejects_invalid_field_types() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia".to_string(),
            options: serde_json::json!({
                "graph_structural_transport": {
                    "constraint_filter": {
                        "timeout_secs": "fast"
                    }
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let Err(error) = build_graph_structural_flight_transport_client(
        &repository,
        GraphStructuralRouteKind::ConstraintFilter,
    ) else {
        panic!("invalid timeout type must fail");
    };
    assert!(
        error
            .to_string()
            .contains("Julia plugin field `timeout_secs` must be an unsigned integer"),
        "unexpected error: {error}"
    );
}
