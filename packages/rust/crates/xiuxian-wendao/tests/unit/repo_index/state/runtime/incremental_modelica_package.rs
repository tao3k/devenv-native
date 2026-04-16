use super::support::*;

#[tokio::test]
async fn prepare_incremental_analysis_merges_root_package_modelica_source_changes() {
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
        id: "incremental-modelica-package-doc-bail".to_string(),
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
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\n  import Modelica.Math;\n  annotation(Documentation(info = \"doc\"));\nend DemoLib;\n",
    )
    .unwrap_or_else(|error| panic!("rewrite root package source: {error}"));
    commit_all(tempdir.path(), "root package Modelica change");
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
        .unwrap_or_else(|error| panic!("prepare root package Modelica change: {error}"));

    let Some(PreparedIncrementalAnalysis::Analysis(analysis)) = prepared else {
        panic!("expected incremental analysis merge for root package Modelica change");
    };
    assert!(
        analysis.imports.iter().any(|import| {
            import.path == "package.mo"
                && import.module_id == "repo:incremental-modelica-package-doc-bail:module:DemoLib"
                && import.import_name == "Math"
                && import.target_package == "Modelica"
                && import.source_module == "Modelica.Math"
        }),
        "imports: {:?}",
        analysis.imports
    );
    assert!(
        analysis
            .docs
            .iter()
            .any(|doc| doc.path == "package.mo#annotation.documentation"),
        "docs: {:?}",
        analysis.docs
    );
    assert!(
        analysis.relations.iter().any(|relation| {
            relation.kind == crate::analyzers::RelationKind::Documents
                && relation.source_id
                    == "repo:incremental-modelica-package-doc-bail:doc:package.mo#annotation.documentation"
                && relation.target_id
                    == "repo:incremental-modelica-package-doc-bail:module:DemoLib"
        }),
        "relations: {:?}",
        analysis.relations
    );
    guard.kill();
}

#[tokio::test]
async fn prepare_incremental_analysis_returns_none_for_root_package_modelica_rename() {
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
        id: "incremental-modelica-root-rename".to_string(),
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
        tempdir.path().join("package.mo"),
        "within ;\npackage RenamedLib\nend RenamedLib;\n",
    )
    .unwrap_or_else(|error| panic!("rewrite root package name: {error}"));
    commit_all(tempdir.path(), "root package rename");
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
        .unwrap_or_else(|error| panic!("prepare root package rename: {error}"));

    assert!(
        prepared.is_none(),
        "root package rename should stay on full-analysis fallback"
    );
    guard.kill();
}
