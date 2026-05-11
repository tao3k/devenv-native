use super::support::{
    Arc, PathBuf, PluginRegistry, PreparedIncrementalAnalysis, RegisteredRepository,
    RepoSourceKind, RepoSyncResult, RepositoryPluginConfig, RepositoryRefreshPolicy,
    RuntimeRustPlugin, SearchPlaneService, analyze_registered_repository_with_registry, commit_all,
    fs, init_git_repository, new_coordinator_with_registry,
};

#[tokio::test]
async fn prepare_incremental_analysis_reuses_cached_analysis_for_ast_equivalent_generic_rust_source_churn()
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
        id: "incremental-generic-rust-ast-equivalent".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("rust".to_string())],
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
    let baseline =
        analyze_registered_repository_with_registry(&repository, tempdir.path(), registry.as_ref())
            .unwrap_or_else(|error| panic!("seed Rust analysis cache: {error}"));

    fs::write(
        tempdir.path().join("src/lib.rs"),
        "fn solve(x: i32) -> i32 {\n    // semantic no-op\n    x + 1\n}\n",
    )
    .unwrap_or_else(|error| panic!("rewrite Rust source: {error}"));
    commit_all(tempdir.path(), "ast equivalent rust change");
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
        .unwrap_or_else(|error| panic!("prepare generic Rust reuse: {error}"));

    let analysis = match prepared {
        Some(PreparedIncrementalAnalysis::Analysis(analysis)) => analysis,
        Some(PreparedIncrementalAnalysis::RefreshOnly) => {
            panic!("expected cached analysis reuse, got refresh-only incremental result")
        }
        None => panic!("expected cached analysis reuse, got full-analysis fallback"),
    };
    assert_eq!(analysis.modules, baseline.modules);
    assert_eq!(analysis.symbols, baseline.symbols);
    assert_eq!(analysis.imports, baseline.imports);
    assert_eq!(analysis.examples, baseline.examples);
    assert_eq!(analysis.docs, baseline.docs);
    assert_eq!(analysis.relations, baseline.relations);
}

#[tokio::test]
async fn prepare_incremental_analysis_returns_none_for_semantic_generic_rust_source_change() {
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
        id: "incremental-generic-rust-semantic-change".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("rust".to_string())],
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
            .unwrap_or_else(|error| panic!("seed Rust analysis cache: {error}"));

    fs::write(
        tempdir.path().join("src/lib.rs"),
        "fn solve(x: i32, y: i32) -> i32 {\n    x + y\n}\n",
    )
    .unwrap_or_else(|error| panic!("rewrite Rust source: {error}"));
    commit_all(tempdir.path(), "semantic rust change");
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
        .unwrap_or_else(|error| panic!("prepare generic Rust semantic change: {error}"));

    assert!(
        prepared.is_none(),
        "semantic Rust change should stay on full-analysis fallback"
    );
}
