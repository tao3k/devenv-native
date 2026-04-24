#[test]
fn julia_plugin_artifact_resolution_keeps_transport_diagnostics() {
    let selector = julia_deployment_artifact_selector();
    let runtime = LinkGraphJuliaRerankRuntimeConfig {
        base_url: Some("http://127.0.0.1:8088".to_string()),
        route: Some(DEFAULT_JULIA_RERANK_FLIGHT_ROUTE.to_string()),
        health_route: Some("/healthz".to_string()),
        schema_version: Some("v1".to_string()),
        timeout_secs: Some(15),
        service_mode: Some("stream".to_string()),
        search_config_path: Some(DEFAULT_JULIA_SEARCH_EXAMPLE_CONFIG_PATH.to_string()),
        vector_weight: Some(0.2),
        similarity_weight: Some(0.8),
    };

    let Some(payload) = resolve_julia_plugin_artifact_payload_for_selector(&selector, &runtime)
    else {
        panic!("artifact payload");
    };

    assert_eq!(payload.plugin_id, selector.plugin_id);
    assert_eq!(payload.artifact_id, selector.artifact_id);
    assert_eq!(
        payload.selected_transport,
        Some(xiuxian_wendao_core::transport::PluginTransportKind::ArrowFlight)
    );
    assert_eq!(payload.fallback_from, None);
    assert_eq!(payload.fallback_reason, None);
}

#[test]
fn julia_plugin_artifact_rendering_serializes_resolved_payload()
-> Result<(), Box<dyn std::error::Error>> {
    let selector = julia_deployment_artifact_selector();
    let runtime = LinkGraphJuliaRerankRuntimeConfig {
        base_url: Some("http://127.0.0.1:8088".to_string()),
        route: Some(DEFAULT_JULIA_RERANK_FLIGHT_ROUTE.to_string()),
        health_route: Some("/healthz".to_string()),
        schema_version: Some("v1".to_string()),
        timeout_secs: Some(15),
        service_mode: Some("stream".to_string()),
        search_config_path: Some(DEFAULT_JULIA_SEARCH_EXAMPLE_CONFIG_PATH.to_string()),
        vector_weight: Some(0.2),
        similarity_weight: Some(0.8),
    };

    let Some(rendered) = render_julia_plugin_artifact_toml_for_selector(&selector, &runtime)?
    else {
        panic!("rendered payload");
    };

    assert!(rendered.contains("plugin_id = \"xiuxian-wendao-julia\""));
    assert!(rendered.contains("artifact_id = \"deployment\""));
    assert!(rendered.contains("selected_transport = \"arrow_flight\""));

    Ok(())
}
