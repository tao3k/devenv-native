use super::support::*;

#[tokio::test]
async fn prepare_incremental_analysis_merges_parameter_bearing_leaf_modelica_source_changes() {
    let (base_url, mut guard) = spawn_wendaosearch_modelica_parser_summary_service().await;
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
        id: "incremental-modelica-semantic-change".to_string(),
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
        "within DemoLib;\nmodel PI\n  parameter Real k = 1;\n  parameter Real Ti = 0.1;\nend PI;\n",
    )
    .unwrap_or_else(|error| panic!("rewrite leaf Modelica source: {error}"));
    commit_all(tempdir.path(), "semantic Modelica change");
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
        .unwrap_or_else(|error| panic!("prepare parameter-bearing Modelica merge: {error}"));

    let Some(PreparedIncrementalAnalysis::Analysis(analysis)) = prepared else {
        panic!("expected incremental analysis merge for parameter-bearing Modelica change");
    };
    assert!(
        baseline
            .symbols
            .iter()
            .any(|symbol| symbol.qualified_name == "DemoLib.PI" && symbol.line_end == Some(4)),
        "baseline symbols: {:?}",
        baseline.symbols
    );
    assert!(
        analysis
            .symbols
            .iter()
            .any(|symbol| symbol.qualified_name == "DemoLib.PI" && symbol.line_end == Some(5)),
        "symbols: {:?}",
        analysis.symbols
    );
    assert!(
        analysis.imports.is_empty(),
        "imports: {:?}",
        analysis.imports
    );
    assert_eq!(analysis.docs, baseline.docs);
    guard.kill();
}

#[tokio::test]
async fn prepare_incremental_analysis_merges_leaf_modelica_source_changes() {
    let (base_url, mut guard) = spawn_wendaosearch_modelica_parser_summary_service().await;
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
        id: "incremental-modelica-leaf-merge".to_string(),
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
        "within DemoLib;\nmodel PID\nend PID;\n",
    )
    .unwrap_or_else(|error| panic!("rewrite leaf Modelica source: {error}"));
    commit_all(tempdir.path(), "leaf Modelica semantic change");
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
        .unwrap_or_else(|error| panic!("prepare leaf Modelica merge: {error}"));

    let Some(PreparedIncrementalAnalysis::Analysis(analysis)) = prepared else {
        panic!("expected incremental analysis merge for leaf Modelica source change");
    };
    assert!(
        analysis
            .symbols
            .iter()
            .any(|symbol| symbol.qualified_name == "DemoLib.PID"),
        "symbols: {:?}",
        analysis.symbols
    );
    assert!(
        analysis
            .symbols
            .iter()
            .all(|symbol| symbol.qualified_name != "DemoLib.PI"),
        "symbols: {:?}",
        analysis.symbols
    );
    assert!(
        analysis.imports.is_empty(),
        "imports: {:?}",
        analysis.imports
    );
    guard.kill();
}
