#[test]
fn capability_manifest_build_client_returns_none_without_config() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Id("julia-code-parser".to_string())],
        ..RegisteredRepository::default()
    };

    let client = build_julia_capability_manifest_flight_transport_client(&repository)
        .unwrap_or_else(|error| panic!("missing config should be ignored: {error}"));
    assert!(client.is_none());
}

#[test]
fn capability_manifest_build_client_reads_nested_options() {
    let repository = configured_repository(serde_json::json!({
        "capability_manifest_transport": {
            "base_url": "http://127.0.0.1:9105",
            "health_route": "/ready",
            "timeout_secs": 21
        }
    }));

    let client = build_julia_capability_manifest_flight_transport_client(&repository)
        .unwrap_or_else(|error| panic!("manifest config should parse: {error}"))
        .unwrap_or_else(|| panic!("manifest client should exist"));

    assert_eq!(client.flight_base_url(), "http://127.0.0.1:9105");
    assert_eq!(
        client.flight_route(),
        JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE
    );
    assert_eq!(
        client.selection().selected_transport,
        PluginTransportKind::ArrowFlight
    );
}

#[test]
fn capability_manifest_build_client_rejects_invalid_field_types() {
    let repository = configured_repository(serde_json::json!({
        "capability_manifest_transport": {
            "timeout_secs": "fast"
        }
    }));

    let Err(error) = build_julia_capability_manifest_flight_transport_client(&repository) else {
        panic!("invalid timeout type must fail");
    };
    assert!(
        error
            .to_string()
            .contains("Julia plugin field `timeout_secs` must be an unsigned integer"),
        "unexpected error: {error}"
    );
}
