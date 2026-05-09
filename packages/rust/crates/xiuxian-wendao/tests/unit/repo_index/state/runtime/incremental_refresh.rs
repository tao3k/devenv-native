use super::support::{
    Arc, PathBuf, PreparedIncrementalAnalysis, RegisteredRepository, RepoSourceKind,
    RepoSyncResult, RepositoryPluginConfig, RepositoryRefreshPolicy, SearchPlaneService,
    analyze_registered_repository_with_registry, bootstrap_builtin_registry, commit_all, fs,
    init_git_repository, julia_parser_summary_plugin_config, new_coordinator_with_registry,
    spawn_wendaosearch_julia_parser_summary_service,
};

#[tokio::test]
async fn prepare_incremental_analysis_returns_refresh_only_for_non_code_revision_churn() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    fs::write(
        tempdir.path().join("Project.toml"),
        "name = \"FixturePkg\"\n",
    )
    .unwrap_or_else(|error| panic!("write Project.toml: {error}"));
    fs::write(
        tempdir.path().join("src/FixturePkg.jl"),
        "module FixturePkg\nend\n",
    )
    .unwrap_or_else(|error| panic!("write Julia source: {error}"));
    fs::write(tempdir.path().join("notes.txt"), "first note\n")
        .unwrap_or_else(|error| panic!("write notes: {error}"));
    commit_all(tempdir.path(), "initial");
    let previous_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover previous revision"));

    fs::write(tempdir.path().join("notes.txt"), "second note\n")
        .unwrap_or_else(|error| panic!("rewrite notes: {error}"));
    commit_all(tempdir.path(), "non-code");
    let current_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover current revision"));

    let repository = RegisteredRepository {
        id: "incremental-refresh-only".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("julia-code-parser".to_string())],
    };
    let registry = Arc::new(
        bootstrap_builtin_registry().unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    );
    let coordinator =
        new_coordinator_with_registry(SearchPlaneService::new(PathBuf::from(".")), registry);

    let prepared = coordinator
        .prepare_incremental_analysis(
            &repository,
            &RepoSyncResult {
                repo_id: repository.id.clone(),
                source_kind: RepoSourceKind::LocalCheckout,
                checkout_path: tempdir.path().display().to_string(),
                revision: Some(current_revision),
                ..RepoSyncResult::default()
            },
            Some(previous_revision.as_str()),
        )
        .unwrap_or_else(|error| panic!("prepare incremental refresh-only: {error}"));

    assert!(matches!(
        prepared,
        Some(PreparedIncrementalAnalysis::RefreshOnly)
    ));
}

#[tokio::test]
async fn prepare_incremental_analysis_merges_leaf_julia_source_changes() {
    let (base_url, mut guard) = spawn_wendaosearch_julia_parser_summary_service();
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    fs::write(
        tempdir.path().join("Project.toml"),
        "name = \"FixturePkg\"\n",
    )
    .unwrap_or_else(|error| panic!("write Project.toml: {error}"));
    fs::write(
        tempdir.path().join("src/FixturePkg.jl"),
        "module FixturePkg\ninclude(\"leaf.jl\")\nend\n",
    )
    .unwrap_or_else(|error| panic!("write root Julia source: {error}"));
    fs::write(tempdir.path().join("src/leaf.jl"), "alpha() = 1\n")
        .unwrap_or_else(|error| panic!("write leaf Julia source: {error}"));
    commit_all(tempdir.path(), "initial");
    let previous_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover previous revision"));

    let repository = RegisteredRepository {
        id: "incremental-leaf-merge".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![julia_parser_summary_plugin_config(&base_url)],
    };
    let registry = Arc::new(
        bootstrap_builtin_registry().unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    );
    let coordinator = new_coordinator_with_registry(
        SearchPlaneService::new(PathBuf::from(".")),
        Arc::clone(&registry),
    );
    analyze_registered_repository_with_registry(&repository, tempdir.path(), registry.as_ref())
        .unwrap_or_else(|error| panic!("seed analysis cache: {error}"));

    fs::write(
        tempdir.path().join("src/leaf.jl"),
        "alpha() = 2\nbeta() = 3\n",
    )
    .unwrap_or_else(|error| panic!("rewrite leaf Julia source: {error}"));
    commit_all(tempdir.path(), "leaf change");
    let current_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover current revision"));

    let prepared = coordinator
        .prepare_incremental_analysis(
            &repository,
            &RepoSyncResult {
                repo_id: repository.id.clone(),
                source_kind: RepoSourceKind::LocalCheckout,
                checkout_path: tempdir.path().display().to_string(),
                revision: Some(current_revision),
                ..RepoSyncResult::default()
            },
            Some(previous_revision.as_str()),
        )
        .unwrap_or_else(|error| panic!("prepare incremental merge: {error}"));

    let Some(PreparedIncrementalAnalysis::Analysis(analysis)) = prepared else {
        panic!("expected incremental analysis merge");
    };
    assert!(analysis.symbols.iter().any(|symbol| {
        symbol.qualified_name == "FixturePkg.alpha" && symbol.path == "src/leaf.jl"
    }));
    assert!(analysis.symbols.iter().any(|symbol| {
        symbol.qualified_name == "FixturePkg.beta" && symbol.path == "src/leaf.jl"
    }));
    guard.kill();
}
