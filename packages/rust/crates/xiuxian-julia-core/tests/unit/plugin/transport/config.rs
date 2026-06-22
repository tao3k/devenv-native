#[test]
fn build_julia_flight_transport_client_returns_none_without_inline_config() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Id("julia-code-parser".to_string())],
        ..RegisteredRepository::default()
    };

    let client = match build_julia_flight_transport_client(&repository) {
        Ok(client) => client,
        Err(error) => panic!("expected missing inline config to be ignored: {error}"),
    };
    assert!(client.is_none());
}

#[test]
fn build_julia_flight_transport_client_reads_nested_flight_transport_options() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "flight_transport": {
                    "base_url": "http://127.0.0.1:8081",
                    "route": "/analysis",
                    "health_route": "/ready",
                    "timeout_secs": 30,
                    "max_in_flight_requests": 7
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let client = match build_julia_flight_transport_client(&repository) {
        Ok(Some(client)) => client,
        Ok(None) => panic!("expected inline Julia Arrow Flight transport config"),
        Err(error) => panic!("expected nested config to build successfully: {error}"),
    };

    assert_eq!(client.flight_base_url(), "http://127.0.0.1:8081");
    assert_eq!(client.flight_route(), "/analysis");
    assert_eq!(
        client.selection().selected_transport,
        PluginTransportKind::ArrowFlight
    );

    let binding = build_flight_transport_binding(&repository)
        .unwrap_or_else(|error| panic!("binding should parse: {error}"))
        .unwrap_or_else(|| panic!("binding should exist"));
    assert_eq!(binding.endpoint.max_in_flight_requests, Some(7));
}

#[test]
fn build_julia_flight_transport_client_rejects_invalid_field_types() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "flight_transport": {
                    "timeout_secs": "fast"
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let Err(error) = build_julia_flight_transport_client(&repository) else {
        panic!("expected invalid timeout type to fail");
    };
    assert!(
        error
            .to_string()
            .contains("Julia plugin field `timeout_secs` must be an unsigned integer"),
        "unexpected error: {error}"
    );
}

#[test]
fn build_julia_flight_transport_client_honors_enabled_false() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "flight_transport": {
                    "enabled": false,
                    "base_url": "http://127.0.0.1:8081"
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let client = match build_julia_flight_transport_client(&repository) {
        Ok(client) => client,
        Err(error) => panic!("expected disabled config to be ignored: {error}"),
    };
    assert!(client.is_none());
}

#[test]
fn build_julia_flight_transport_client_rejects_zero_in_flight_budget() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "flight_transport": {
                    "max_in_flight_requests": 0
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let Err(error) = build_julia_flight_transport_client(&repository) else {
        panic!("zero in-flight budget must fail");
    };
    assert!(
        error.to_string().contains("max_in_flight_requests"),
        "unexpected error: {error}"
    );
}
