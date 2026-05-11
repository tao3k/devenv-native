use super::support::{
    Arc, PathBuf, PreparedIncrementalAnalysis, RegisteredRepository, RepoSourceKind,
    RepoSyncResult, RepositoryRefreshPolicy, SearchPlaneService,
    analyze_registered_repository_with_registry, bootstrap_builtin_registry, commit_all, fs,
    init_git_repository, mixed_julia_modelica_plugin_configs, new_coordinator_with_registry,
    spawn_wendaosearch_julia_parser_summary_service,
    spawn_wendaosearch_modelica_parser_summary_service,
};

#[tokio::test]
async fn prepare_incremental_analysis_reuses_cached_analysis_for_ast_equivalent_mixed_julia_modelica_julia_source_churn()
 {
    let (julia_base_url, mut julia_guard) = spawn_wendaosearch_julia_parser_summary_service();
    let (modelica_base_url, mut modelica_guard) =
        spawn_wendaosearch_modelica_parser_summary_service();
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    fs::write(tempdir.path().join("Project.toml"), "name = \"MixedPkg\"\n")
        .unwrap_or_else(|error| panic!("write Project.toml: {error}"));
    fs::write(
        tempdir.path().join("src/MixedPkg.jl"),
        "module MixedPkg\ninclude(\"leaf.jl\")\nend\n",
    )
    .unwrap_or_else(|error| panic!("write root Julia source: {error}"));
    fs::write(tempdir.path().join("src/leaf.jl"), "alpha() = 1\n")
        .unwrap_or_else(|error| panic!("write leaf Julia source: {error}"));
    fs::write(
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )
    .unwrap_or_else(|error| panic!("write root package: {error}"));
    fs::write(
        tempdir.path().join("PI.mo"),
        "within DemoLib;\nmodel PI\n  parameter Real k = 1;\nend PI;\n",
    )
    .unwrap_or_else(|error| panic!("write leaf Modelica source: {error}"));
    commit_all(tempdir.path(), "initial");
    let previous_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover previous revision"));

    let repository = RegisteredRepository {
        id: "incremental-mixed-julia-modelica-julia".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: mixed_julia_modelica_plugin_configs(&julia_base_url, &modelica_base_url),
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
        "alpha() = 1\n# semantic no-op\n",
    )
    .unwrap_or_else(|error| panic!("rewrite leaf Julia source: {error}"));
    commit_all(tempdir.path(), "ast equivalent mixed Julia change");
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
        .unwrap_or_else(|error| panic!("prepare mixed Julia reuse: {error}"));

    let Some(PreparedIncrementalAnalysis::Analysis(analysis)) = prepared else {
        panic!("expected cached analysis reuse for mixed Julia AST-equivalent change");
    };
    assert_eq!(analysis.modules, baseline.modules);
    assert_eq!(analysis.symbols, baseline.symbols);
    assert_eq!(analysis.imports, baseline.imports);
    assert_eq!(analysis.examples, baseline.examples);
    assert_eq!(analysis.docs, baseline.docs);
    assert_eq!(analysis.relations, baseline.relations);
    julia_guard.kill();
    modelica_guard.kill();
}

#[tokio::test]
async fn prepare_incremental_analysis_reuses_cached_analysis_for_ast_equivalent_mixed_julia_modelica_modelica_source_churn()
 {
    let (julia_base_url, mut julia_guard) = spawn_wendaosearch_julia_parser_summary_service();
    let (modelica_base_url, mut modelica_guard) =
        spawn_wendaosearch_modelica_parser_summary_service();
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    fs::write(tempdir.path().join("Project.toml"), "name = \"MixedPkg\"\n")
        .unwrap_or_else(|error| panic!("write Project.toml: {error}"));
    fs::write(
        tempdir.path().join("src/MixedPkg.jl"),
        "module MixedPkg\ninclude(\"leaf.jl\")\nend\n",
    )
    .unwrap_or_else(|error| panic!("write root Julia source: {error}"));
    fs::write(tempdir.path().join("src/leaf.jl"), "alpha() = 1\n")
        .unwrap_or_else(|error| panic!("write leaf Julia source: {error}"));
    fs::write(
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )
    .unwrap_or_else(|error| panic!("write root package: {error}"));
    fs::write(
        tempdir.path().join("PI.mo"),
        "within DemoLib;\nmodel PI\n  parameter Real k = 1;\nend PI;\n",
    )
    .unwrap_or_else(|error| panic!("write leaf Modelica source: {error}"));
    commit_all(tempdir.path(), "initial");
    let previous_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover previous revision"));

    let repository = RegisteredRepository {
        id: "incremental-mixed-julia-modelica-modelica".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: mixed_julia_modelica_plugin_configs(&julia_base_url, &modelica_base_url),
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
        tempdir.path().join("PI.mo"),
        "within DemoLib;\nmodel PI\n  parameter Real k = 1;\nend PI;\n// semantic no-op\n",
    )
    .unwrap_or_else(|error| panic!("rewrite leaf Modelica source: {error}"));
    commit_all(tempdir.path(), "ast equivalent mixed Modelica change");
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
        .unwrap_or_else(|error| panic!("prepare mixed Modelica reuse: {error}"));

    let Some(PreparedIncrementalAnalysis::Analysis(analysis)) = prepared else {
        panic!("expected cached analysis reuse for mixed Modelica AST-equivalent change");
    };
    assert_eq!(analysis.modules, baseline.modules);
    assert_eq!(analysis.symbols, baseline.symbols);
    assert_eq!(analysis.imports, baseline.imports);
    assert_eq!(analysis.examples, baseline.examples);
    assert_eq!(analysis.docs, baseline.docs);
    assert_eq!(analysis.relations, baseline.relations);
    julia_guard.kill();
    modelica_guard.kill();
}
