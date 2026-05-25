#[tokio::test]
async fn demo_capability_manifest_live_proof_covers_fetch_preflight_binding_and_plugin_preflight() {
    if !local_wendaosearch_package_available() {
        eprintln!(
            "skipping real WendaoSearch capability-manifest live proof; set WENDAOSEARCH_PACKAGE_DIR"
        );
        return;
    }

    let port = reserve_real_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let _service = spawn_real_wendaosearch_demo_capability_manifest_service(port);
    let repository = live_capability_manifest_repository(&base_url);

    await_live_step(
        wait_for_service_ready_with_attempts(&format!("http://127.0.0.1:{port}"), 600),
        LIVE_SERVICE_STARTUP_TIMEOUT_SECS,
        "wait for real WendaoSearch capability-manifest service",
    )
    .await
        .unwrap_or_else(|error| {
            panic!("wait for real WendaoSearch capability-manifest service: {error}")
        });

    let rows = await_live_step(
        fetch_julia_plugin_capability_manifest_rows_for_repository(
            &repository,
            &[JuliaPluginCapabilityManifestRequestRow {
                plugin_id: JULIA_PLUGIN_ID.into(),
                repository_id: repository.id.clone().into(),
                capability_filter: None,
                include_disabled: true.into(),
            }],
        ),
        LIVE_REQUEST_TIMEOUT_SECS,
        "real WendaoSearch capability-manifest fetch",
    )
    .await
    .unwrap_or_else(|error| {
        panic!("real WendaoSearch capability-manifest fetch should succeed: {error}")
    });
    assert_live_capability_manifest_rows(rows.as_slice());
    assert_live_manifest_discovery_clients(&repository, base_url.as_str());
    assert_live_plugin_preflight(&repository);
}

fn assert_live_capability_manifest_rows(rows: &[JuliaPluginCapabilityManifestRow]) {
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(|row| row.plugin_id.as_str() == JULIA_PLUGIN_ID));
    assert!(
        rows.iter()
            .any(|row| row.capability_id.as_str() == JULIA_CAPABILITY_MANIFEST_CAPABILITY_ID)
    );
}

fn assert_live_manifest_discovery_clients(repository: &RegisteredRepository, base_url: &str) {
    let rows = validate_julia_capability_manifest_preflight_for_repository(repository)
        .unwrap_or_else(|error| {
            panic!("real WendaoSearch capability-manifest preflight should succeed: {error}")
        })
        .unwrap_or_else(|| panic!("manifest transport should be discovered"));

    assert!(
        rows.iter()
            .any(|row| row.capability_id.as_str() == JULIA_CAPABILITY_MANIFEST_CAPABILITY_ID)
    );

    let binding = discover_julia_graph_structural_binding_from_manifest_for_repository(
        repository,
        GraphStructuralRouteKind::StructuralRerank,
    )
    .unwrap_or_else(|error| {
        panic!("manifest discovery should derive a graph-structural binding: {error}")
    })
    .unwrap_or_else(|| panic!("graph-structural binding should exist"));

    assert_eq!(
        binding.endpoint.base_url.as_deref(),
        Some(base_url)
    );
    assert_eq!(
        binding.endpoint.route.as_deref(),
        Some("/graph/structural/rerank")
    );

    let rerank_client = build_graph_structural_flight_transport_client(
        repository,
        GraphStructuralRouteKind::StructuralRerank,
    )
    .unwrap_or_else(|error| panic!("manifest fallback should parse rerank route: {error}"))
    .unwrap_or_else(|| panic!("manifest fallback rerank client should exist"));
    let filter_client = build_graph_structural_flight_transport_client(
        repository,
        GraphStructuralRouteKind::ConstraintFilter,
    )
    .unwrap_or_else(|error| panic!("manifest fallback should parse filter route: {error}"))
    .unwrap_or_else(|| panic!("manifest fallback filter client should exist"));

    assert_eq!(rerank_client.flight_base_url(), base_url);
    assert_eq!(rerank_client.flight_route(), "/graph/structural/rerank");
    assert_eq!(filter_client.flight_base_url(), base_url);
    assert_eq!(filter_client.flight_route(), "/graph/structural/filter");
}

fn assert_live_plugin_preflight(repository: &RegisteredRepository) {
    let temp = tempdir().unwrap_or_else(|error| panic!("create temp repo: {error}"));
    fs::create_dir_all(temp.path().join("src"))
        .unwrap_or_else(|error| panic!("create src directory: {error}"));
    fs::write(
        temp.path().join("Project.toml"),
        "name = \"DemoPkg\"\nversion = \"0.1.0\"\n",
    )
    .unwrap_or_else(|error| panic!("write Project.toml: {error}"));
    fs::write(
        temp.path().join("src").join("DemoPkg.jl"),
        "module DemoPkg\n\nexport greet\n\ngreet() = :ok\n\nend\n",
    )
    .unwrap_or_else(|error| panic!("write root Julia module: {error}"));

    let repository_with_path = RegisteredRepository {
        path: Some(temp.path().to_path_buf()),
        ..repository.clone()
    };
    let context = AnalysisContext {
        repository: repository_with_path,
        repository_root: temp.path().to_path_buf(),
    };

    JuliaRepoIntelligencePlugin
        .preflight_repository(&context, temp.path())
        .unwrap_or_else(|error| {
            panic!("repository preflight with live capability manifest should succeed: {error}")
        });
}
