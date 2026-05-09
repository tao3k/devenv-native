use super::{
    Arc, BTreeMap, ExampleRecord, GatewayState, ImportKind, ImportRecord, LocalCheckoutMetadata,
    MaterializedRepo, ModuleRecord, PathBuf, RegisteredRepository, RepoCodeDocument,
    RepoDriftState, RepoLifecycleState, RepoSourceKind, RepoSymbolKind, RepositoryAnalysisOutput,
    RepositoryPluginConfig, RepositoryRefreshPolicy, SearchMaintenancePolicy,
    SearchManifestKeyspace, SearchPlaneService, StudioState, SymbolRecord, SyncMode, UiConfig,
    UiRepoProjectConfig, bootstrap_builtin_registry, build_repository_analysis_cache_key,
    commit_all, configured_repository, discover_checkout_metadata, init_git_repository,
    resolve_registered_repository_source, store_cached_repository_analysis,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct CachedRepoSearchProbe {
    pub(super) value: String,
}

pub(super) struct ImportGatewayFixture {
    pub(super) _temp_dir: tempfile::TempDir,
    pub(super) state: Arc<GatewayState>,
}

pub(super) fn unique_repo_gateway_keyspace(
    label: &str,
    root: &std::path::Path,
) -> SearchManifestKeyspace {
    SearchManifestKeyspace::new(format!(
        "xiuxian:test:repo_gateway:{label}:{}",
        blake3::hash(root.to_string_lossy().as_bytes()).to_hex()
    ))
}

pub(super) fn normalized_gateway_analysis_keys() -> (
    xiuxian_wendao::analyzers::RepositoryAnalysisCacheKey,
    xiuxian_wendao::analyzers::RepositoryAnalysisCacheKey,
) {
    let source = MaterializedRepo {
        checkout_root: PathBuf::from("/tmp/gateway-repo-search-normalized"),
        mirror_root: None,
        mirror_revision: Some("mirror-1".to_string()),
        tracking_revision: Some("tracking-1".to_string()),
        last_fetched_at: None,
        drift_state: RepoDriftState::NotApplicable,
        mirror_state: RepoLifecycleState::NotApplicable,
        checkout_state: RepoLifecycleState::Validated,
        source_kind: RepoSourceKind::LocalCheckout,
    };
    let metadata = Some(LocalCheckoutMetadata {
        revision: Some("rev-1".to_string()),
        remote_url: None,
    });
    let first_analysis_key = build_repository_analysis_cache_key(
        &RegisteredRepository {
            id: "gateway/normalized".to_string(),
            path: Some(PathBuf::from("/tmp/gateway-repo-search-normalized")),
            url: None,
            git_ref: None,
            refresh: RepositoryRefreshPolicy::Fetch,
            plugins: vec![
                RepositoryPluginConfig::Id("ast-grep".to_string()),
                RepositoryPluginConfig::Id("julia-code-parser".to_string()),
                RepositoryPluginConfig::Config {
                    id: "modelica".to_string(),
                    options: serde_json::json!({
                        "mode": "parser-summary"
                    }),
                },
            ],
        },
        &source,
        metadata.as_ref(),
    );
    let second_analysis_key = build_repository_analysis_cache_key(
        &RegisteredRepository {
            id: "gateway/normalized".to_string(),
            path: Some(PathBuf::from("/tmp/gateway-repo-search-normalized")),
            url: None,
            git_ref: None,
            refresh: RepositoryRefreshPolicy::Fetch,
            plugins: vec![
                RepositoryPluginConfig::Config {
                    id: "modelica".to_string(),
                    options: serde_json::json!({
                        "mode": "doc-surface"
                    }),
                },
                RepositoryPluginConfig::Id("ast-grep".to_string()),
                RepositoryPluginConfig::Id("julia-code-parser".to_string()),
                RepositoryPluginConfig::Id("ast-grep".to_string()),
            ],
        },
        &source,
        metadata.as_ref(),
    );

    (first_analysis_key, second_analysis_key)
}

pub(super) async fn sample_repo_entity_service(
    keyspace: &str,
) -> (tempfile::TempDir, SearchPlaneService) {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let service = SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        SearchManifestKeyspace::new(keyspace),
        SearchMaintenancePolicy::default(),
    );
    let analysis = sample_analysis("alpha/repo", "solve", "Shows solve");
    let documents = sample_documents("solve", 10);
    service
        .publish_repo_entities_with_revision("alpha/repo", &analysis, &documents, Some("rev-1"))
        .await
        .unwrap_or_else(|error| panic!("publish repo entities: {error}"));
    (temp_dir, service)
}

fn sample_analysis(
    repo_id: &str,
    symbol_name: &str,
    example_summary: &str,
) -> RepositoryAnalysisOutput {
    let mut attributes = std::collections::BTreeMap::new();
    attributes.insert("arity".to_string(), "0".to_string());
    RepositoryAnalysisOutput {
        modules: vec![ModuleRecord {
            repo_id: repo_id.to_string(),
            module_id: "module:BaseModelica".to_string(),
            qualified_name: "BaseModelica".to_string(),
            path: "src/BaseModelica.jl".to_string(),
        }],
        symbols: vec![SymbolRecord {
            repo_id: repo_id.to_string(),
            symbol_id: format!("symbol:{symbol_name}"),
            module_id: Some("module:BaseModelica".to_string()),
            name: symbol_name.to_string(),
            qualified_name: format!("BaseModelica.{symbol_name}"),
            kind: RepoSymbolKind::Function,
            path: "src/BaseModelica.jl".to_string(),
            line_start: Some(7),
            line_end: Some(9),
            signature: Some(format!("{symbol_name}()")),
            audit_status: Some("verified".to_string()),
            verification_state: Some("verified".to_string()),
            attributes,
        }],
        examples: vec![ExampleRecord {
            repo_id: repo_id.to_string(),
            example_id: "example:solve".to_string(),
            title: "Solve example".to_string(),
            path: "examples/solve.jl".to_string(),
            summary: Some(example_summary.to_string()),
        }],
        imports: vec![ImportRecord {
            repo_id: repo_id.to_string(),
            module_id: "module:BaseModelica".to_string(),
            path: "src/BaseModelica.jl".to_string(),
            import_name: "solve".to_string(),
            target_package: "SciMLBase".to_string(),
            source_module: "BaseModelica".to_string(),
            kind: ImportKind::Reexport,
            line_start: None,
            resolved_id: Some(format!("symbol:{symbol_name}")),
            attributes: BTreeMap::new(),
        }],
        ..RepositoryAnalysisOutput::default()
    }
}

fn sample_documents(symbol_name: &str, source_modified_unix_ms: u64) -> Vec<RepoCodeDocument> {
    vec![
        RepoCodeDocument {
            path: "src/BaseModelica.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from(format!(
                "module BaseModelica\n{symbol_name}() = nothing\nend\n"
            )),
            size_bytes: 48,
            modified_unix_ms: source_modified_unix_ms,
        },
        RepoCodeDocument {
            path: "examples/solve.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from("using BaseModelica\nsolve()\n"),
            size_bytes: 28,
            modified_unix_ms: 10,
        },
    ]
}

pub(super) async fn sample_repo_entity_gateway_fixture(keyspace: &str) -> ImportGatewayFixture {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let registry = Arc::new(
        bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap builtin registry: {error:?}")),
    );
    let studio = StudioState::new_with_bootstrap_ui_config_and_search_plane_root(
        registry,
        temp_dir.path().join("search_plane").join(keyspace),
    );
    let analysis = sample_analysis("alpha/repo", "solve", "Shows solve");
    let documents = sample_documents("solve", 10);
    let state = Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(studio),
    });
    state
        .studio
        .search_plane
        .publish_repo_entities_with_revision("alpha/repo", &analysis, &documents, Some("rev-1"))
        .await
        .unwrap_or_else(|error| panic!("publish repo entities: {error}"));

    ImportGatewayFixture {
        _temp_dir: temp_dir,
        state,
    }
}

pub(super) fn sample_import_gateway_fixture(keyspace: &str) -> ImportGatewayFixture {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let repo_root = temp_dir.path().join("projectionpkg");
    std::fs::create_dir_all(repo_root.join("src"))
        .unwrap_or_else(|error| panic!("create src dir: {error}"));
    std::fs::create_dir_all(repo_root.join("examples"))
        .unwrap_or_else(|error| panic!("create examples dir: {error}"));
    std::fs::write(
        repo_root.join("Project.toml"),
        r#"name = "ProjectionPkg"
uuid = "12345678-1234-1234-1234-123456789abc"
version = "0.1.0"

[deps]
Reexport = "189a3867-3050-52da-a836-e630ba90ab69"
SciMLBase = "0bca4576-84f4-4d90-8ffe-ffa030f20462"
"#,
    )
    .unwrap_or_else(|error| panic!("write project: {error}"));
    std::fs::write(
        repo_root.join("src").join("ProjectionPkg.jl"),
        r"module ProjectionPkg

using Reexport
@reexport using SciMLBase

export solve

solve(problem) = problem

end
",
    )
    .unwrap_or_else(|error| panic!("write source: {error}"));
    std::fs::write(
        repo_root.join("examples").join("basic.jl"),
        "using ProjectionPkg\nsolve(1)\n",
    )
    .unwrap_or_else(|error| panic!("write example: {error}"));
    initialize_git_repository(&repo_root);

    let registry = Arc::new(
        bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap builtin registry: {error:?}")),
    );
    let studio = StudioState::new_with_bootstrap_ui_config_and_search_plane_root(
        Arc::clone(&registry),
        temp_dir.path().join("search_plane").join(keyspace),
    );
    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: Vec::new(),
        repo_projects: vec![UiRepoProjectConfig {
            id: "sciml/imports".to_string(),
            root: Some(repo_root.to_string_lossy().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["julia-code-parser".to_string()],
        }],
    });
    prime_import_analysis_cache(&studio);

    ImportGatewayFixture {
        _temp_dir: temp_dir,
        state: Arc::new(GatewayState {
            index: None,
            signal_tx: None,
            webhook_url: None,
            studio: Arc::new(studio),
        }),
    }
}

fn prime_import_analysis_cache(studio: &StudioState) {
    let repository = configured_repository(studio, "sciml/imports")
        .unwrap_or_else(|error| panic!("resolve configured repository: {error:?}"));
    let repository_source = resolve_registered_repository_source(
        &repository,
        studio.project_root.as_path(),
        SyncMode::Status,
    )
    .unwrap_or_else(|error| panic!("resolve repository source: {error:?}"));
    let checkout_metadata = discover_checkout_metadata(repository_source.checkout_root.as_path());
    let cache_key = build_repository_analysis_cache_key(
        &repository,
        &repository_source,
        checkout_metadata.as_ref(),
    );
    store_cached_repository_analysis(cache_key, &sample_import_analysis("sciml/imports"))
        .unwrap_or_else(|error| panic!("store repository analysis cache: {error:?}"));
}

fn sample_import_analysis(repo_id: &str) -> RepositoryAnalysisOutput {
    RepositoryAnalysisOutput {
        modules: vec![ModuleRecord {
            repo_id: repo_id.to_string(),
            module_id: "module:ProjectionPkg".to_string(),
            qualified_name: "ProjectionPkg".to_string(),
            path: "src/ProjectionPkg.jl".to_string(),
        }],
        symbols: vec![SymbolRecord {
            repo_id: repo_id.to_string(),
            symbol_id: "symbol:solve".to_string(),
            module_id: Some("module:ProjectionPkg".to_string()),
            name: "solve".to_string(),
            qualified_name: "ProjectionPkg.solve".to_string(),
            kind: RepoSymbolKind::Function,
            path: "src/ProjectionPkg.jl".to_string(),
            line_start: Some(7),
            line_end: Some(7),
            signature: Some("solve(problem)".to_string()),
            audit_status: None,
            verification_state: None,
            attributes: BTreeMap::new(),
        }],
        imports: vec![ImportRecord {
            repo_id: repo_id.to_string(),
            module_id: "module:ProjectionPkg".to_string(),
            path: "src/ProjectionPkg.jl".to_string(),
            import_name: "SciMLBase".to_string(),
            target_package: "SciMLBase".to_string(),
            source_module: "SciMLBase".to_string(),
            kind: ImportKind::Reexport,
            line_start: Some(4),
            resolved_id: None,
            attributes: BTreeMap::new(),
        }],
        ..RepositoryAnalysisOutput::default()
    }
}

fn initialize_git_repository(repo_root: &std::path::Path) {
    init_git_repository(repo_root);
    commit_all(repo_root, "initial import");
}
