use super::support::{
    Arc, PathBuf, PreparedIncrementalAnalysis, RegisteredRepository, RepoSourceKind,
    RepoSyncResult, RepositoryRefreshPolicy, SearchPlaneService,
    analyze_registered_repository_with_registry, bootstrap_builtin_registry, commit_all, fs,
    init_git_repository, modelica_parser_summary_plugin_config, new_coordinator_with_registry,
    spawn_wendaosearch_modelica_parser_summary_service,
};

#[tokio::test]
async fn prepare_incremental_analysis_merges_import_bearing_leaf_modelica_source_changes() {
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
        "within DemoLib;\nmodel PI\nend PI;\n",
    )
    .unwrap_or_else(|error| panic!("write leaf Modelica source: {error}"));
    commit_all(tempdir.path(), "initial");
    let previous_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover previous revision"));

    let repository = RegisteredRepository {
        id: "incremental-modelica-import-merge".to_string(),
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
    analyze_registered_repository_with_registry(&repository, tempdir.path(), registry.as_ref())
        .unwrap_or_else(|error| panic!("seed analysis cache: {error}"));

    fs::write(
        tempdir.path().join("PI.mo"),
        "within DemoLib;\nmodel PI\n  import Modelica.Math;\nend PI;\n",
    )
    .unwrap_or_else(|error| panic!("rewrite import-bearing Modelica source: {error}"));
    commit_all(tempdir.path(), "import-bearing Modelica change");
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
        .unwrap_or_else(|error| panic!("prepare import-bearing Modelica merge: {error}"));

    let Some(PreparedIncrementalAnalysis::Analysis(analysis)) = prepared else {
        panic!("expected incremental analysis merge for import-bearing Modelica change");
    };
    assert!(
        analysis.imports.iter().any(|import| {
            import.path == "PI.mo"
                && import.module_id == "repo:incremental-modelica-import-merge:module:DemoLib"
                && import.import_name == "Math"
                && import.target_package == "Modelica"
                && import.source_module == "Modelica.Math"
        }),
        "imports: {:?}",
        analysis.imports
    );
    guard.kill();
}

#[tokio::test]
async fn prepare_incremental_analysis_returns_none_for_documentation_annotation_modelica_source_change()
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
        "within DemoLib;\nmodel PI\nend PI;\n",
    )
    .unwrap_or_else(|error| panic!("write leaf Modelica source: {error}"));
    commit_all(tempdir.path(), "initial");
    let previous_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover previous revision"));

    let repository = RegisteredRepository {
        id: "incremental-modelica-doc-bail".to_string(),
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
    analyze_registered_repository_with_registry(&repository, tempdir.path(), registry.as_ref())
        .unwrap_or_else(|error| panic!("seed analysis cache: {error}"));

    fs::write(
        tempdir.path().join("PI.mo"),
        "within DemoLib;\nmodel PI\n  annotation(Documentation(info = \"doc\"));\nend PI;\n",
    )
    .unwrap_or_else(|error| panic!("rewrite documentation Modelica source: {error}"));
    commit_all(tempdir.path(), "documentation annotation Modelica change");
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
        .unwrap_or_else(|error| {
            panic!("prepare documentation annotation Modelica change: {error}")
        });

    assert!(
        prepared.is_none(),
        "documentation annotation Modelica change should stay on full-analysis fallback"
    );
    guard.kill();
}
