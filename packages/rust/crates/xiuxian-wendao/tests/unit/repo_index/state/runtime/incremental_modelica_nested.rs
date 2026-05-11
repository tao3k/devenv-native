use super::support::{
    Arc, PathBuf, PreparedIncrementalAnalysis, RegisteredRepository, RepoSourceKind,
    RepoSyncResult, RepositoryPluginConfig, RepositoryRefreshPolicy, SearchPlaneService,
    analyze_registered_repository_with_registry, bootstrap_builtin_registry, commit_all,
    ensure_linked_modelica_parser_summary_service, fs, init_git_repository,
    new_coordinator_with_registry,
};

#[tokio::test]
async fn prepare_incremental_analysis_reuses_cached_analysis_for_ast_equivalent_nested_modelica_package_source_churn()
 {
    ensure_linked_modelica_parser_summary_service()
        .unwrap_or_else(|error| panic!("linked Modelica parser-summary service: {error}"));
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::write(
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )
    .unwrap_or_else(|error| panic!("write root package: {error}"));
    fs::create_dir_all(tempdir.path().join("Blocks"))
        .unwrap_or_else(|error| panic!("create Blocks dir: {error}"));
    fs::write(
        tempdir.path().join("Blocks/package.mo"),
        "within DemoLib;\npackage Blocks\n  import Modelica.Math;\nend Blocks;\n",
    )
    .unwrap_or_else(|error| panic!("write nested package: {error}"));
    commit_all(tempdir.path(), "initial");
    let previous_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover previous revision"));

    let repository = RegisteredRepository {
        id: "incremental-modelica-nested-package-ast-equivalent".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
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
        tempdir.path().join("Blocks/package.mo"),
        "within DemoLib;\npackage Blocks\n  // semantic no-op\n  import Modelica.Math;\nend Blocks;\n",
    )
    .unwrap_or_else(|error| panic!("rewrite nested package: {error}"));
    commit_all(tempdir.path(), "ast equivalent nested package change");
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
        .unwrap_or_else(|error| panic!("prepare nested package incremental reuse: {error}"));

    let Some(PreparedIncrementalAnalysis::Analysis(analysis)) = prepared else {
        panic!("expected cached analysis reuse for AST-equivalent nested package change");
    };
    assert_eq!(analysis.modules, baseline.modules);
    assert_eq!(analysis.symbols, baseline.symbols);
    assert_eq!(analysis.imports, baseline.imports);
    assert_eq!(analysis.examples, baseline.examples);
    assert_eq!(analysis.docs, baseline.docs);
    assert_eq!(analysis.relations, baseline.relations);
}

#[tokio::test]
async fn prepare_incremental_analysis_merges_nested_package_modelica_source_changes() {
    ensure_linked_modelica_parser_summary_service()
        .unwrap_or_else(|error| panic!("linked Modelica parser-summary service: {error}"));
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::write(
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )
    .unwrap_or_else(|error| panic!("write root package: {error}"));
    fs::create_dir_all(tempdir.path().join("Blocks"))
        .unwrap_or_else(|error| panic!("create Blocks dir: {error}"));
    fs::write(
        tempdir.path().join("Blocks/package.mo"),
        "within DemoLib;\npackage Blocks\nend Blocks;\n",
    )
    .unwrap_or_else(|error| panic!("write nested package: {error}"));
    commit_all(tempdir.path(), "initial");
    let previous_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover previous revision"));

    let repository = RegisteredRepository {
        id: "incremental-modelica-nested-package-merge".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
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
        tempdir.path().join("Blocks/package.mo"),
        "within DemoLib;\npackage Blocks\n  import Modelica.Math;\n  annotation(Documentation(info = \"doc\"));\nend Blocks;\n",
    )
    .unwrap_or_else(|error| panic!("rewrite nested package: {error}"));
    commit_all(tempdir.path(), "nested package Modelica change");
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
        .unwrap_or_else(|error| panic!("prepare nested package Modelica change: {error}"));

    let Some(PreparedIncrementalAnalysis::Analysis(analysis)) = prepared else {
        panic!("expected incremental analysis merge for nested package Modelica change");
    };
    assert!(
        analysis.imports.iter().any(|import| {
            import.path == "Blocks/package.mo"
                && import.module_id
                    == "repo:incremental-modelica-nested-package-merge:module:DemoLib.Blocks"
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
            .any(|doc| doc.path == "Blocks/package.mo#annotation.documentation"),
        "docs: {:?}",
        analysis.docs
    );
    assert!(
        analysis.relations.iter().any(|relation| {
            relation.source_id
                == "repo:incremental-modelica-nested-package-merge:doc:Blocks/package.mo#annotation.documentation"
                && relation.target_id
                    == "repo:incremental-modelica-nested-package-merge:module:DemoLib.Blocks"
        }),
        "relations: {:?}",
        analysis.relations
    );
}

#[tokio::test]
async fn prepare_incremental_analysis_returns_none_for_nested_package_modelica_declaration_change()
{
    ensure_linked_modelica_parser_summary_service()
        .unwrap_or_else(|error| panic!("linked Modelica parser-summary service: {error}"));
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::write(
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )
    .unwrap_or_else(|error| panic!("write root package: {error}"));
    fs::create_dir_all(tempdir.path().join("Blocks"))
        .unwrap_or_else(|error| panic!("create Blocks dir: {error}"));
    fs::write(
        tempdir.path().join("Blocks/package.mo"),
        "within DemoLib;\npackage Blocks\nend Blocks;\n",
    )
    .unwrap_or_else(|error| panic!("write nested package: {error}"));
    commit_all(tempdir.path(), "initial");
    let previous_revision = xiuxian_git_repo::discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover previous revision"));

    let repository = RegisteredRepository {
        id: "incremental-modelica-nested-package-fallback".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
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
        tempdir.path().join("Blocks/package.mo"),
        "within DemoLib;\npackage Blocks\n  model Controller\n  end Controller;\nend Blocks;\n",
    )
    .unwrap_or_else(|error| panic!("rewrite nested package: {error}"));
    commit_all(tempdir.path(), "nested package declaration change");
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
        .unwrap_or_else(|error| panic!("prepare nested package declaration change: {error}"));

    assert!(
        prepared.is_none(),
        "nested package declaration change should stay on full-analysis fallback"
    );
}
