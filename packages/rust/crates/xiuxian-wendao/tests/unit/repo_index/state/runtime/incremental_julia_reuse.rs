use super::support::{
    Arc, PathBuf, PreparedIncrementalAnalysis, RegisteredRepository, RepoSourceKind,
    RepoSyncResult, RepositoryRefreshPolicy, SearchPlaneService,
    analyze_registered_repository_with_registry, bootstrap_builtin_registry, commit_all, fs,
    init_git_repository, julia_parser_summary_plugin_config, new_coordinator_with_registry,
    spawn_wendaosearch_julia_parser_summary_service,
};

#[tokio::test]
async fn prepare_incremental_analysis_reuses_cached_analysis_for_example_churn() {
    let (base_url, mut guard) = spawn_wendaosearch_julia_parser_summary_service();
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    fs::create_dir_all(tempdir.path().join("examples"))
        .unwrap_or_else(|error| panic!("create examples: {error}"));
    fs::write(
        tempdir.path().join("Project.toml"),
        "name = \"FixturePkg\"\n",
    )
    .unwrap_or_else(|error| panic!("write Project.toml: {error}"));
    fs::write(
        tempdir.path().join("src/FixturePkg.jl"),
        "module FixturePkg\nalpha() = 1\nend\n",
    )
    .unwrap_or_else(|error| panic!("write root Julia source: {error}"));
    fs::write(
        tempdir.path().join("examples/demo.jl"),
        "using FixturePkg\nalpha()\n",
    )
    .unwrap_or_else(|error| panic!("write example Julia source: {error}"));
    commit_all(tempdir.path(), "initial");
    let previous_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover previous revision"));

    let repository = RegisteredRepository {
        id: "incremental-example-reuse".to_string(),
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
    let baseline =
        analyze_registered_repository_with_registry(&repository, tempdir.path(), registry.as_ref())
            .unwrap_or_else(|error| panic!("seed analysis cache: {error}"));

    fs::write(
        tempdir.path().join("examples/demo.jl"),
        "using FixturePkg\nalpha()\nalpha()\n",
    )
    .unwrap_or_else(|error| panic!("rewrite example Julia source: {error}"));
    commit_all(tempdir.path(), "example change");
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
        .unwrap_or_else(|error| panic!("prepare incremental example reuse: {error}"));

    let Some(PreparedIncrementalAnalysis::Analysis(analysis)) = prepared else {
        panic!("expected cached analysis reuse");
    };
    assert_eq!(analysis.modules, baseline.modules);
    assert_eq!(analysis.symbols, baseline.symbols);
    assert_eq!(analysis.examples, baseline.examples);
    guard.kill();
}

#[tokio::test]
async fn prepare_incremental_analysis_reuses_cached_analysis_for_ast_equivalent_julia_source_churn()
{
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
    fs::write(tempdir.path().join("src/leaf.jl"), "alpha() = 1\n\n")
        .unwrap_or_else(|error| panic!("write leaf Julia source: {error}"));
    commit_all(tempdir.path(), "initial");
    let previous_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover previous revision"));

    let repository = RegisteredRepository {
        id: "incremental-ast-equivalent".to_string(),
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
    let baseline =
        analyze_registered_repository_with_registry(&repository, tempdir.path(), registry.as_ref())
            .unwrap_or_else(|error| panic!("seed analysis cache: {error}"));

    fs::write(
        tempdir.path().join("src/leaf.jl"),
        "alpha() = 1\n# trailing comment should stay AST-equivalent\n",
    )
    .unwrap_or_else(|error| panic!("rewrite leaf Julia source: {error}"));
    commit_all(tempdir.path(), "ast equivalent leaf change");
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
        .unwrap_or_else(|error| panic!("prepare incremental ast-equivalent reuse: {error}"));

    let Some(PreparedIncrementalAnalysis::Analysis(analysis)) = prepared else {
        panic!("expected cached analysis reuse for AST-equivalent change");
    };
    assert_eq!(analysis.modules, baseline.modules);
    assert_eq!(analysis.symbols, baseline.symbols);
    assert_eq!(analysis.examples, baseline.examples);
    guard.kill();
}
