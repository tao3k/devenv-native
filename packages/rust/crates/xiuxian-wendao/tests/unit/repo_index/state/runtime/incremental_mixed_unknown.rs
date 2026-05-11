use super::support::{
    Arc, PathBuf, PluginRegistry, RegisteredRepository, RepoSourceKind, RepoSyncResult,
    RepositoryRefreshPolicy, RuntimeModelicaPlugin, RuntimeRustPlugin, SearchPlaneService,
    analyze_registered_repository_with_registry, commit_all, fs, init_git_repository,
    mixed_modelica_unknown_plugin_configs, mixed_rust_unknown_plugin_configs,
    new_coordinator_with_registry,
};

#[tokio::test]
async fn prepare_incremental_analysis_returns_none_for_ast_equivalent_mixed_rust_unknown_plugin_source_churn()
 {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    fs::write(
        tempdir.path().join("src/lib.rs"),
        "fn solve(x: i32) -> i32 {\n    x + 1\n}\n",
    )
    .unwrap_or_else(|error| panic!("write Rust source: {error}"));
    commit_all(tempdir.path(), "initial");
    let previous_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover previous revision"));

    let repository = RegisteredRepository {
        id: "incremental-mixed-rust-unknown".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: mixed_rust_unknown_plugin_configs(),
    };
    let mut registry = PluginRegistry::new();
    registry
        .register(RuntimeRustPlugin)
        .unwrap_or_else(|error| panic!("register Rust runtime plugin: {error}"));
    let registry = Arc::new(registry);
    let coordinator = new_coordinator_with_registry(
        SearchPlaneService::new(PathBuf::from(".")),
        Arc::clone(&registry),
    );
    let _baseline =
        analyze_registered_repository_with_registry(&repository, tempdir.path(), registry.as_ref())
            .unwrap_or_else(|error| panic!("seed mixed analysis cache: {error}"));

    fs::write(
        tempdir.path().join("src/lib.rs"),
        "fn solve(x: i32) -> i32 {\n    // semantic no-op\n    x + 1\n}\n",
    )
    .unwrap_or_else(|error| panic!("rewrite Rust source: {error}"));
    commit_all(tempdir.path(), "ast equivalent mixed Rust unknown change");
    let current_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover current revision"));

    let prepared = coordinator
        .prepare_incremental_analysis(
            &repository,
            &RepoSyncResult {
                repo_id: repository.id.clone().into(),
                source_kind: RepoSourceKind::LocalCheckout,
                checkout_path: tempdir.path().display().to_string(),
                revision: Some(current_revision),
                ..RepoSyncResult::default()
            },
            Some(previous_revision.as_str()),
        )
        .unwrap_or_else(|error| panic!("prepare mixed Rust unknown fallback: {error}"));

    assert!(
        prepared.is_none(),
        "unknown plugin mix should stay on full-analysis fallback"
    );
}

#[tokio::test]
async fn prepare_incremental_analysis_returns_none_for_ast_equivalent_mixed_modelica_unknown_plugin_source_churn()
 {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::write(
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )
    .unwrap_or_else(|error| panic!("write root package: {error}"));
    fs::write(
        tempdir.path().join("PI.mo"),
        "within DemoLib;\nmodel PI\n  parameter Real k = 1;\nend PI;\n",
    )
    .unwrap_or_else(|error| panic!("write Modelica source: {error}"));
    commit_all(tempdir.path(), "initial");
    let previous_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover previous revision"));

    let repository = RegisteredRepository {
        id: "incremental-mixed-modelica-unknown".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: mixed_modelica_unknown_plugin_configs(),
    };
    let mut registry = PluginRegistry::new();
    registry
        .register(RuntimeModelicaPlugin)
        .unwrap_or_else(|error| panic!("register Modelica runtime plugin: {error}"));
    let registry = Arc::new(registry);
    let coordinator = new_coordinator_with_registry(
        SearchPlaneService::new(PathBuf::from(".")),
        Arc::clone(&registry),
    );
    let _baseline =
        analyze_registered_repository_with_registry(&repository, tempdir.path(), registry.as_ref())
            .unwrap_or_else(|error| panic!("seed mixed analysis cache: {error}"));

    fs::write(
        tempdir.path().join("PI.mo"),
        "within DemoLib;\nmodel PI\n  parameter Real k = 1;\nend PI;\n// semantic no-op\n",
    )
    .unwrap_or_else(|error| panic!("rewrite Modelica source: {error}"));
    commit_all(
        tempdir.path(),
        "ast equivalent mixed Modelica unknown change",
    );
    let current_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover current revision"));

    let prepared = coordinator
        .prepare_incremental_analysis(
            &repository,
            &RepoSyncResult {
                repo_id: repository.id.clone().into(),
                source_kind: RepoSourceKind::LocalCheckout,
                checkout_path: tempdir.path().display().to_string(),
                revision: Some(current_revision),
                ..RepoSyncResult::default()
            },
            Some(previous_revision.as_str()),
        )
        .unwrap_or_else(|error| panic!("prepare mixed Modelica unknown fallback: {error}"));

    assert!(
        prepared.is_none(),
        "unknown plugin mix should stay on full-analysis fallback"
    );
}
