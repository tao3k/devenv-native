use super::support::{
    Arc, PathBuf, PreparedIncrementalAnalysis, RegisteredRepository, RepoSourceKind,
    RepoSyncResult, RepositoryRefreshPolicy, SearchPlaneService,
    analyze_registered_repository_with_registry, bootstrap_builtin_registry, commit_all, fs,
    init_git_repository, modelica_parser_summary_plugin_config, new_coordinator_with_registry,
    spawn_wendaosearch_modelica_parser_summary_service,
};

#[tokio::test]
async fn prepare_incremental_analysis_reuses_cached_analysis_for_ast_equivalent_modelica_source_churn()
 {
    let (base_url, mut guard) = spawn_wendaosearch_modelica_parser_summary_service();
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
    .unwrap_or_else(|error| panic!("write leaf Modelica source: {error}"));
    commit_all(tempdir.path(), "initial");
    let previous_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover previous revision"));

    let repository = RegisteredRepository {
        id: "incremental-modelica-ast-equivalent".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![modelica_parser_summary_plugin_config(&base_url)],
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
    commit_all(tempdir.path(), "ast equivalent Modelica change");
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
        .unwrap_or_else(|error| panic!("prepare incremental Modelica reuse: {error}"));

    let Some(PreparedIncrementalAnalysis::Analysis(analysis)) = prepared else {
        panic!("expected cached analysis reuse for AST-equivalent Modelica change");
    };
    assert_eq!(analysis.modules, baseline.modules);
    assert_eq!(analysis.symbols, baseline.symbols);
    assert_eq!(analysis.imports, baseline.imports);
    assert_eq!(analysis.examples, baseline.examples);
    assert_eq!(analysis.docs, baseline.docs);
    assert_eq!(analysis.relations, baseline.relations);
    guard.kill();
}

#[tokio::test]
async fn prepare_incremental_analysis_reuses_cached_analysis_for_ast_equivalent_modelica_package_source_churn()
 {
    let (base_url, mut guard) = spawn_wendaosearch_modelica_parser_summary_service();
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
    .unwrap_or_else(|error| panic!("write leaf Modelica source: {error}"));
    commit_all(tempdir.path(), "initial");
    let previous_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover previous revision"));

    let repository = RegisteredRepository {
        id: "incremental-modelica-package-ast-equivalent".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![modelica_parser_summary_plugin_config(&base_url)],
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
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\n// semantic no-op\nend DemoLib;\n",
    )
    .unwrap_or_else(|error| panic!("rewrite package.mo: {error}"));
    commit_all(tempdir.path(), "ast equivalent package.mo change");
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
        .unwrap_or_else(|error| panic!("prepare package.mo incremental reuse: {error}"));

    let Some(PreparedIncrementalAnalysis::Analysis(analysis)) = prepared else {
        panic!("expected cached analysis reuse for AST-equivalent package.mo change");
    };
    assert_eq!(analysis.modules, baseline.modules);
    assert_eq!(analysis.symbols, baseline.symbols);
    assert_eq!(analysis.imports, baseline.imports);
    assert_eq!(analysis.examples, baseline.examples);
    assert_eq!(analysis.docs, baseline.docs);
    assert_eq!(analysis.relations, baseline.relations);
    guard.kill();
}
