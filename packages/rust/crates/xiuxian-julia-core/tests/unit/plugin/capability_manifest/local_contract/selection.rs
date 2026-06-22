#[test]
fn capability_manifest_selects_graph_structural_binding_by_variant() {
    let rows = vec![
        JuliaPluginCapabilityManifestRow {
            plugin_id: JULIA_PLUGIN_ID.into(),
            capability_id: JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID.into(),
            capability_variant: Some("structural_rerank".into()),
            transport_kind: "arrow_flight".into(),
            base_url: "http://127.0.0.1:8815".into(),
            route: "/graph/structural/rerank".into(),
            health_route: Some("/healthz".into()),
            schema_version: "v0-draft".into(),
            timeout_secs: Some(15_u64.into()),
            enabled: true.into(),
        },
        JuliaPluginCapabilityManifestRow {
            plugin_id: JULIA_PLUGIN_ID.into(),
            capability_id: JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID.into(),
            capability_variant: Some("constraint_filter".into()),
            transport_kind: "arrow_flight".into(),
            base_url: "http://127.0.0.1:8815".into(),
            route: "/graph/structural/filter".into(),
            health_route: Some("/healthz".into()),
            schema_version: "v0-draft".into(),
            timeout_secs: Some(15_u64.into()),
            enabled: true.into(),
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

#[test]
fn capability_manifest_selects_graph_structural_binding_by_alias_variant() {
    let rows = vec![JuliaPluginCapabilityManifestRow {
        plugin_id: JULIA_PLUGIN_ID.into(),
        capability_id: JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID.into(),
        capability_variant: Some("rerank".into()),
        transport_kind: "arrow_flight".into(),
        base_url: "http://127.0.0.1:8815".into(),
        route: "/graph/structural/rerank".into(),
        health_route: Some("/healthz".into()),
        schema_version: "v0-draft".into(),
        timeout_secs: Some(15_u64.into()),
        enabled: true.into(),
    }];

    let binding = graph_structural_binding_from_capability_manifest_rows(
        rows.as_slice(),
        GraphStructuralRouteKind::StructuralRerank,
    )
    .unwrap_or_else(|error| panic!("alias variant should resolve: {error}"))
    .unwrap_or_else(|| panic!("alias variant binding should exist"));
    assert_eq!(
        binding.endpoint.route.as_deref(),
        Some("/graph/structural/rerank")
    );
}

#[test]
fn capability_manifest_selects_graph_structural_binding_by_route_fallback() {
    let rows = vec![JuliaPluginCapabilityManifestRow {
        plugin_id: JULIA_PLUGIN_ID.into(),
        capability_id: JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID.into(),
        capability_variant: Some("other_variant".into()),
        transport_kind: "arrow_flight".into(),
        base_url: "http://127.0.0.1:8815".into(),
        route: "/graph/structural/filter".into(),
        health_route: Some("/healthz".into()),
        schema_version: "v0-draft".into(),
        timeout_secs: Some(15_u64.into()),
        enabled: true.into(),
    }];

    let binding = graph_structural_binding_from_capability_manifest_rows(
        rows.as_slice(),
        GraphStructuralRouteKind::ConstraintFilter,
    )
    .unwrap_or_else(|error| panic!("route fallback should resolve: {error}"))
    .unwrap_or_else(|| panic!("route fallback binding should exist"));
    assert_eq!(
        binding.endpoint.route.as_deref(),
        Some("/graph/structural/filter")
    );
}
