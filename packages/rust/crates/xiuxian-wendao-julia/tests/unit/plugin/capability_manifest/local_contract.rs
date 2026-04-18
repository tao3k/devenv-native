#[test]
fn capability_manifest_build_client_returns_none_without_config() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Id("julia".to_string())],
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

#[test]
fn capability_manifest_request_batch_materializes_rows() {
    let batch = build_julia_plugin_capability_manifest_request_batch(&[
        JuliaPluginCapabilityManifestRequestRow {
            plugin_id: "xiuxian-wendao-julia".to_string(),
            repository_id: "repo-julia".to_string(),
            capability_filter: Some("graph-structural".to_string()),
            include_disabled: true,
        },
    ])
    .unwrap_or_else(|error| panic!("request batch should build: {error}"));

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(batch.schema().fields().len(), 4);
}

#[test]
fn capability_manifest_decode_rows_materializes_bindings_and_variants() {
    let rows = decode_julia_plugin_capability_manifest_rows(&[sample_response_batch()])
        .unwrap_or_else(|error| panic!("response rows should decode: {error}"));

    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[1].capability_variant.as_deref(),
        Some("structural_rerank")
    );

    let binding = rows[0]
        .to_binding()
        .unwrap_or_else(|error| panic!("enabled row should convert into binding: {error}"))
        .unwrap_or_else(|| panic!("enabled row should produce a binding"));
    assert_eq!(binding.selector, rows[0].selector());
    assert_eq!(binding.endpoint.route.as_deref(), Some("/rerank"));
    assert_eq!(binding.contract_version.0, "v1".to_string());

    let disabled_binding = rows[1]
        .to_binding()
        .unwrap_or_else(|error| panic!("disabled row should still validate: {error}"));
    assert!(disabled_binding.is_none());
}

#[test]
fn capability_manifest_response_validation_rejects_unsupported_transport() {
    let batch = RecordBatch::try_new(
        julia_plugin_capability_manifest_response_schema(),
        vec![
            Arc::new(StringArray::from(vec![Some("xiuxian-wendao-julia")])),
            Arc::new(StringArray::from(vec![Some("rerank")])),
            Arc::new(StringArray::from(vec![None::<&str>])),
            Arc::new(StringArray::from(vec![Some("http")])),
            Arc::new(StringArray::from(vec![Some("http://127.0.0.1:8815")])),
            Arc::new(StringArray::from(vec![Some("/rerank")])),
            Arc::new(StringArray::from(vec![Some("/healthz")])),
            Arc::new(StringArray::from(vec![Some(
                JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION,
            )])),
            Arc::new(UInt64Array::from(vec![Some(15)])),
            Arc::new(BooleanArray::from(vec![true])),
        ],
    )
    .unwrap_or_else(|error| panic!("invalid transport batch should build: {error}"));

    let Err(error) = validate_julia_plugin_capability_manifest_response_batches(&[batch]) else {
        panic!("unsupported transport should fail");
    };
    assert!(
        error
            .to_string()
            .contains("unsupported `transport_kind` `http`"),
        "unexpected error: {error}"
    );
}

#[test]
fn capability_manifest_selects_graph_structural_binding_by_variant() {
    let rows = vec![
        JuliaPluginCapabilityManifestRow {
            plugin_id: JULIA_PLUGIN_ID.to_string(),
            capability_id: JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID.to_string(),
            capability_variant: Some("structural_rerank".to_string()),
            transport_kind: "arrow_flight".to_string(),
            base_url: "http://127.0.0.1:8815".to_string(),
            route: "/graph/structural/rerank".to_string(),
            health_route: Some("/healthz".to_string()),
            schema_version: "v0-draft".to_string(),
            timeout_secs: Some(15),
            enabled: true,
        },
        JuliaPluginCapabilityManifestRow {
            plugin_id: JULIA_PLUGIN_ID.to_string(),
            capability_id: JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID.to_string(),
            capability_variant: Some("constraint_filter".to_string()),
            transport_kind: "arrow_flight".to_string(),
            base_url: "http://127.0.0.1:8815".to_string(),
            route: "/graph/structural/filter".to_string(),
            health_route: Some("/healthz".to_string()),
            schema_version: "v0-draft".to_string(),
            timeout_secs: Some(15),
            enabled: true,
        },
    ];

    let binding = graph_structural_binding_from_capability_manifest_rows(
        rows.as_slice(),
        GraphStructuralRouteKind::ConstraintFilter,
    )
    .unwrap_or_else(|error| panic!("constraint-filter variant should resolve: {error}"))
    .unwrap_or_else(|| panic!("constraint-filter binding should exist"));

    assert_eq!(
        binding.endpoint.route.as_deref(),
        Some("/graph/structural/filter")
    );
    assert_eq!(binding.contract_version.0, "v0-draft".to_string());
}
