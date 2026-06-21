use super::{
    ANALYSIS_MARKDOWN_ROUTE, Arc, FlightDescriptor, FlightService, GatewayState, PathBuf, Request,
    SEARCH_SYMBOLS_ROUTE, SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService,
    StudioFlightRoots, UiConfig, UiProjectConfig, build_repo_search_flight_service,
    build_studio_flight_service, build_studio_flight_service_for_roots, build_symbol_index,
    create_dir_all_or_panic, first_ticket, flight_descriptor_path,
    populate_markdown_analysis_headers, populate_search_headers, tempdir_or_panic,
    test_studio_state, write_file_or_panic,
};

#[tokio::test]
async fn build_studio_flight_service_accepts_runtime_studio_providers() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(
        project_root.join("packages/rust/crates/demo/src"),
        "project fixture dirs should build",
    );
    write_file_or_panic(
        project_root.join("packages/rust/crates/demo/src/lib.rs"),
        "pub struct AlphaService;\npub fn alpha_handler() {}\n",
        "project fixture file should write",
    );

    let mut studio = test_studio_state(project_root.join("studio-flight-service"));
    studio.project_root = project_root.clone();
    studio.config_root = project_root.clone();
    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec!["packages".to_string()],
        }],
        repo_projects: Vec::new(),
    });
    let warmed_index = build_symbol_index(
        studio.project_root.as_path(),
        studio.config_root.as_path(),
        studio.configured_projects().as_slice(),
    );
    studio.symbol_index_coordinator.set_ready_index_for_test(
        studio.configured_projects().as_slice(),
        Arc::clone(&studio.symbol_index),
        warmed_index,
    );
    let state = Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(studio),
    });

    let search_plane = Arc::new(SearchPlaneService::with_paths(
        project_root,
        storage_root,
        SearchManifestKeyspace::new("xiuxian:test:studio-flight-service"),
        SearchMaintenancePolicy::default(),
    ));
    let flight_service = build_studio_flight_service(search_plane, state, "v2", 3)
        .unwrap_or_else(|error| panic!("studio flight service should build: {error}"));
    let descriptor = FlightDescriptor::new_path(
        flight_descriptor_path(SEARCH_SYMBOLS_ROUTE)
            .unwrap_or_else(|error| panic!("descriptor path: {error}")),
    );
    let mut request = Request::new(descriptor);
    populate_search_headers(request.metadata_mut(), "alpha", 5);

    let response = flight_service
        .get_flight_info(request)
        .await
        .unwrap_or_else(|error| {
            panic!("studio flight service should resolve symbols route: {error}")
        });
    let ticket = first_ticket(&response.into_inner(), "symbols route");

    assert_eq!(ticket, SEARCH_SYMBOLS_ROUTE);
}

#[tokio::test]
async fn build_studio_flight_service_for_roots_accepts_runtime_studio_providers() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(
        project_root.join("packages/rust/crates/demo/src"),
        "project fixture dirs should build",
    );
    write_file_or_panic(
        project_root.join("packages/rust/crates/demo/src/lib.rs"),
        "pub struct AlphaService;\npub fn alpha_handler() {}\n",
        "project fixture file should write",
    );
    write_file_or_panic(
        project_root.join("wendao.toml"),
        r#"
[sources.projects.kernel]
root = "."
dirs = ["packages"]
"#,
        "wendao.toml should write",
    );

    let search_plane = Arc::new(SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root,
        SearchManifestKeyspace::new("xiuxian:test:studio-flight-service-roots"),
        SearchMaintenancePolicy::default(),
    ));
    let flight_service = build_studio_flight_service_for_roots(
        search_plane,
        StudioFlightRoots::new(project_root.clone(), project_root.clone()),
        "v2",
        3,
    )
    .unwrap_or_else(|error| panic!("studio flight service should build from roots: {error}"));
    let descriptor = FlightDescriptor::new_path(
        flight_descriptor_path(SEARCH_SYMBOLS_ROUTE)
            .unwrap_or_else(|error| panic!("descriptor path: {error}")),
    );
    let mut request = Request::new(descriptor);
    populate_search_headers(request.metadata_mut(), "alpha", 5);

    let response = flight_service
        .get_flight_info(request)
        .await
        .unwrap_or_else(|error| {
            panic!("studio flight service should resolve symbols route: {error}")
        });
    let ticket = first_ticket(&response.into_inner(), "symbols route");

    assert_eq!(ticket, SEARCH_SYMBOLS_ROUTE);
}

#[tokio::test]
async fn build_studio_flight_service_for_roots_accepts_markdown_analysis_routes() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(project_root.join("docs"), "project docs dir should build");
    create_dir_all_or_panic(
        project_root.join("kernel/docs"),
        "project-scoped docs dir should build",
    );
    write_file_or_panic(
        project_root.join("docs/analysis.md"),
        "# Analysis Kernel\n\n## Inputs\n- [ ] Parse markdown\n",
        "project markdown fixture should write",
    );
    write_file_or_panic(
        project_root.join("kernel/docs/analysis.md"),
        "# Analysis Kernel\n\n## Inputs\n- [ ] Parse markdown\n",
        "project-scoped markdown fixture should write",
    );
    write_file_or_panic(
        project_root.join("wendao.toml"),
        r#"
[sources.projects.kernel]
root = "."
dirs = ["docs"]
"#,
        "wendao.toml should write",
    );

    let search_plane = Arc::new(SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root,
        SearchManifestKeyspace::new("xiuxian:test:flight-studio-service-roots-markdown"),
        SearchMaintenancePolicy::default(),
    ));
    let flight_service = build_studio_flight_service_for_roots(
        search_plane,
        StudioFlightRoots::new(project_root.clone(), project_root.clone()),
        "v2",
        3,
    )
    .unwrap_or_else(|error| panic!("studio flight service should build from roots: {error}"));
    let descriptor = FlightDescriptor::new_path(
        flight_descriptor_path(ANALYSIS_MARKDOWN_ROUTE)
            .unwrap_or_else(|error| panic!("descriptor path: {error}")),
    );
    let mut request = Request::new(descriptor);
    populate_markdown_analysis_headers(request.metadata_mut(), "kernel/docs/analysis.md");

    let response = flight_service
        .get_flight_info(request)
        .await
        .unwrap_or_else(|error| {
            panic!("studio flight service should resolve markdown analysis route: {error}")
        });
    let ticket = first_ticket(&response.into_inner(), "markdown analysis route");

    assert_eq!(ticket, ANALYSIS_MARKDOWN_ROUTE);
}

#[test]
fn build_repo_search_flight_service_accepts_runtime_repo_search_provider() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(&project_root, "project root should build");

    let service = Arc::new(SearchPlaneService::with_paths(
        PathBuf::from(&project_root),
        PathBuf::from(&storage_root),
        SearchManifestKeyspace::new("xiuxian:test:flight-repo-search-service"),
        SearchMaintenancePolicy::default(),
    ));
    let flight_service = build_repo_search_flight_service(service, "v2", 3)
        .unwrap_or_else(|error| panic!("flight service should build: {error}"));

    let _ = flight_service;
}
