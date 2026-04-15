use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use crate::analyzers::cache::{
    RepositorySearchQueryCacheKey, build_repository_analysis_cache_key,
    store_cached_repository_analysis,
};
use crate::analyzers::{
    ExampleRecord, ImportKind, ImportRecord, ModuleRecord, RegisteredRepository, RepoSymbolKind,
    RepositoryAnalysisOutput, RepositoryPluginConfig, RepositoryRefreshPolicy, SymbolRecord,
    bootstrap_builtin_registry, resolve_registered_repository_source,
};
use crate::gateway::studio::router::StudioApiError;
use crate::gateway::studio::router::configured_repository;
use crate::gateway::studio::router::handlers::repo::analysis::search::cache::{
    repository_search_key, with_cached_repo_search_result,
};
use crate::gateway::studio::router::handlers::repo::analysis::search::service::run_repo_import_search;
use crate::gateway::studio::router::{GatewayState, StudioState};
use crate::gateway::studio::test_support::{
    assert_studio_json_snapshot, commit_all, init_git_repository,
};
use crate::gateway::studio::types::{UiConfig, UiRepoProjectConfig};
use crate::query_core::{
    query_repo_entity_example_results_if_published, query_repo_entity_import_results_if_published,
    query_repo_entity_module_results_if_published, query_repo_entity_symbol_results_if_published,
};
use crate::repo_index::RepoCodeDocument;
use crate::search::{
    FuzzySearchOptions, SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService,
    publish_repo_entities,
};
use xiuxian_git_repo::{
    LocalCheckoutMetadata, MaterializedRepo, RepoDriftState, RepoLifecycleState, RepoSourceKind,
    SyncMode, discover_checkout_metadata,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct CachedRepoSearchProbe {
    value: String,
}

fn unique_repo_gateway_keyspace(label: &str, root: &std::path::Path) -> SearchManifestKeyspace {
    SearchManifestKeyspace::new(format!(
        "xiuxian:test:repo_gateway:{label}:{}",
        blake3::hash(root.to_string_lossy().as_bytes()).to_hex()
    ))
}

fn normalized_gateway_analysis_keys() -> (
    crate::analyzers::cache::RepositoryAnalysisCacheKey,
    crate::analyzers::cache::RepositoryAnalysisCacheKey,
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
                RepositoryPluginConfig::Id("julia".to_string()),
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
                RepositoryPluginConfig::Id("julia".to_string()),
                RepositoryPluginConfig::Id("ast-grep".to_string()),
            ],
        },
        &source,
        metadata.as_ref(),
    );

    (first_analysis_key, second_analysis_key)
}

#[tokio::test]
async fn cached_repo_search_result_reuses_hot_query_payload() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let keyspace = unique_repo_gateway_keyspace("cache", temp_dir.path());
    let search_plane = SearchPlaneService::with_test_cache(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        keyspace,
        SearchMaintenancePolicy::default(),
    );
    let load_count = Arc::new(AtomicUsize::new(0));

    let first = with_cached_repo_search_result(
        &search_plane,
        "repo.symbol-search",
        "alpha/repo",
        "solve",
        5,
        {
            let load_count = Arc::clone(&load_count);
            || async move {
                load_count.fetch_add(1, Ordering::SeqCst);
                Ok(CachedRepoSearchProbe {
                    value: "first".to_string(),
                })
            }
        },
    )
    .await
    .unwrap_or_else(|error| panic!("first cached search result: {error:?}"));

    let second = with_cached_repo_search_result(
        &search_plane,
        "repo.symbol-search",
        "alpha/repo",
        "solve",
        5,
        {
            let load_count = Arc::clone(&load_count);
            || async move {
                load_count.fetch_add(1, Ordering::SeqCst);
                Err(StudioApiError::internal(
                    "UNEXPECTED_RELOAD",
                    "cached repo search should not execute loader twice",
                    None,
                ))
            }
        },
    )
    .await
    .unwrap_or_else(|error| panic!("cached repo search hit should succeed: {error:?}"));

    assert_eq!(first, second);
    assert_eq!(first.value, "first");
    assert_eq!(load_count.load(Ordering::SeqCst), 1);
}

#[test]
fn repository_search_key_is_stable_for_normalized_plugin_identity() {
    let (first_analysis_key, second_analysis_key) = normalized_gateway_analysis_keys();

    let first_key = repository_search_key(
        &first_analysis_key,
        "repo.module-search",
        "solve",
        10,
        FuzzySearchOptions::document_search(),
    );
    let second_key = repository_search_key(
        &second_analysis_key,
        "repo.module-search",
        "solve",
        10,
        FuzzySearchOptions::document_search(),
    );

    assert_eq!(first_analysis_key, second_analysis_key);
    assert_eq!(first_key, second_key);
}

#[test]
fn projected_page_search_cache_key_is_stable_for_normalized_plugin_identity() {
    let (first_analysis_key, second_analysis_key) = normalized_gateway_analysis_keys();

    let first_key = RepositorySearchQueryCacheKey::new(
        &first_analysis_key,
        "repo.projected-page-search",
        "solve",
        Some("reference".to_string()),
        FuzzySearchOptions::document_search(),
        10,
    );
    let second_key = RepositorySearchQueryCacheKey::new(
        &second_analysis_key,
        "repo.projected-page-search",
        "solve",
        Some("reference".to_string()),
        FuzzySearchOptions::document_search(),
        10,
    );

    assert_eq!(first_analysis_key, second_analysis_key);
    assert_eq!(first_key, second_key);
}

#[tokio::test]
async fn repo_entity_query_core_returns_none_when_publication_is_not_ready() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let service = SearchPlaneService::with_test_cache(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        unique_repo_gateway_keyspace("entity-module-not-ready", temp_dir.path()),
        SearchMaintenancePolicy::default(),
    );

    let result = query_repo_entity_module_results_if_published(
        &service,
        "alpha/repo",
        "BaseModelica",
        5,
        false,
    )
    .await
    .unwrap_or_else(|error| panic!("query helper should return none: {error:?}"));

    assert!(result.is_none());
}

#[tokio::test]
async fn repo_entity_query_core_module_payload_snapshot() {
    let (_temp_dir, service) =
        sample_repo_entity_service("xiuxian:test:repo_entity_module_payload").await;

    let result = query_repo_entity_module_results_if_published(
        &service,
        "alpha/repo",
        "BaseModelica",
        5,
        true,
    )
    .await
    .unwrap_or_else(|error| panic!("query helper should return module payload: {error:?}"));

    assert_studio_json_snapshot("repo_analysis_module_search_plane_payload", result);
}

#[tokio::test]
async fn repo_entity_query_core_symbol_payload_snapshot() {
    let (_temp_dir, service) =
        sample_repo_entity_service("xiuxian:test:repo_entity_symbol_payload").await;

    let result =
        query_repo_entity_symbol_results_if_published(&service, "alpha/repo", "solve", 5, true)
            .await
            .unwrap_or_else(|error| panic!("query helper should return symbol payload: {error:?}"));

    assert_studio_json_snapshot("repo_analysis_symbol_search_plane_payload", result);
}

#[tokio::test]
async fn repo_entity_query_core_example_payload_snapshot() {
    let (_temp_dir, service) =
        sample_repo_entity_service("xiuxian:test:repo_entity_example_payload").await;

    let result =
        query_repo_entity_example_results_if_published(&service, "alpha/repo", "solve", 5, true)
            .await
            .unwrap_or_else(|error| {
                panic!("query helper should return example payload: {error:?}")
            });

    assert_studio_json_snapshot("repo_analysis_example_search_plane_payload", result);
}

#[tokio::test]
async fn repo_entity_query_core_import_payload_snapshot() {
    let (_temp_dir, service) =
        sample_repo_entity_service("xiuxian:test:repo_entity_import_payload").await;

    let result = query_repo_entity_import_results_if_published(
        &service,
        "alpha/repo",
        Some("SciMLBase".to_string()),
        Some("BaseModelica".to_string()),
        5,
        true,
    )
    .await
    .unwrap_or_else(|error| panic!("query helper should return import payload: {error:?}"));

    assert_studio_json_snapshot("repo_analysis_import_query_core_payload", result);
}

#[tokio::test]
async fn repo_import_search_uses_repo_entity_fast_path_when_publication_ready() {
    let fixture = sample_repo_entity_gateway_fixture("xiuxian:test:repo_import_fast_path").await;

    let result = run_repo_import_search(
        Arc::clone(&fixture.state),
        "alpha/repo".to_string(),
        Some("SciMLBase".to_string()),
        Some("BaseModelica".to_string()),
        5,
    )
    .await
    .unwrap_or_else(|error| {
        panic!("repo import search should resolve through repo entity fast path: {error:?}")
    });

    assert_eq!(result.imports.len(), 1);
    assert_eq!(result.imports[0].target_package, "SciMLBase");
    assert_eq!(result.imports[0].source_module, "BaseModelica");
}

#[test]
fn repo_entity_query_core_error_mapping_preserves_gateway_contract() {
    let error = StudioApiError::internal(
        "REPO_MODULE_SEARCH_FAILED",
        "Repo module search task failed",
        Some("broken repo entity payload".to_string()),
    );

    assert_eq!(error.code(), "REPO_MODULE_SEARCH_FAILED");
    assert_eq!(
        error.status(),
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[tokio::test]
async fn repo_import_search_payload_snapshot() {
    let fixture = sample_import_gateway_fixture("xiuxian:test:repo_import_search_payload");

    let result = run_repo_import_search(
        Arc::clone(&fixture.state),
        "sciml/imports".to_string(),
        Some("SciMLBase".to_string()),
        None,
        10,
    )
    .await
    .unwrap_or_else(|error| panic!("repo import search should resolve: {error:?}"));

    assert_studio_json_snapshot("repo_analysis_import_search_payload", result);
}

async fn sample_repo_entity_service(keyspace: &str) -> (tempfile::TempDir, SearchPlaneService) {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let service = SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        SearchManifestKeyspace::new(keyspace),
        SearchMaintenancePolicy::default(),
    );
    let analysis = sample_analysis("alpha/repo", "solve", "Shows solve");
    let documents = sample_documents("solve", 10);
    publish_repo_entities(&service, "alpha/repo", &analysis, &documents, Some("rev-1"))
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

struct ImportGatewayFixture {
    _temp_dir: tempfile::TempDir,
    state: Arc<GatewayState>,
}

async fn sample_repo_entity_gateway_fixture(keyspace: &str) -> ImportGatewayFixture {
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
    publish_repo_entities(
        &state.studio.search_plane,
        "alpha/repo",
        &analysis,
        &documents,
        Some("rev-1"),
    )
    .await
    .unwrap_or_else(|error| panic!("publish repo entities: {error}"));

    ImportGatewayFixture {
        _temp_dir: temp_dir,
        state,
    }
}

fn sample_import_gateway_fixture(keyspace: &str) -> ImportGatewayFixture {
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
            plugins: vec!["julia".to_string()],
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
