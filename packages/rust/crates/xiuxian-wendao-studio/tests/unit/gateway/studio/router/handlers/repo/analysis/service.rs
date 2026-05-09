use std::collections::BTreeMap;
use std::fs;
use std::sync::Arc;

use crate::contracts::{UiConfig, UiRepoProjectConfig};
use crate::studio::router::handlers::repo::analysis::service::overview::run_repo_overview;
use crate::studio::router::{GatewayState, StudioState};
use xiuxian_wendao::analyzers::{
    ExampleRecord, ModuleRecord, RepoSymbolKind, RepositoryAnalysisOutput, SymbolRecord,
    bootstrap_builtin_registry,
};
use xiuxian_wendao::repo_index::RepoCodeDocument;

#[tokio::test]
async fn run_repo_overview_returns_zero_summary_for_search_only_repository() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let search_plane_root = temp.path().join("search-plane");
    let studio = StudioState::new_with_bootstrap_ui_config_and_search_plane_root(
        Arc::new(
            bootstrap_builtin_registry()
                .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
        ),
        search_plane_root,
    );
    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: Vec::new(),
        repo_projects: vec![UiRepoProjectConfig {
            id: "lance".to_string(),
            root: Some(temp.path().display().to_string()),
            url: Some("https://github.com/lance-format/lance".to_string()),
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });
    let state = Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(studio),
    });

    let overview = run_repo_overview(Arc::clone(&state), "lance".to_string())
        .await
        .unwrap_or_else(|error| panic!("search-only repo overview should succeed: {error:?}"));

    assert_eq!(overview.repo_id, "lance");
    assert_eq!(overview.display_name, "lance");
    assert_eq!(overview.revision, None);
    assert_eq!(overview.module_count, 0);
    assert_eq!(overview.symbol_count, 0);
    assert_eq!(overview.example_count, 0);
    assert_eq!(overview.doc_count, 0);
    assert_eq!(overview.hierarchical_uri.as_deref(), Some("repo://lance"));
    assert_eq!(
        overview.hierarchy,
        Some(vec!["repo".to_string(), "lance".to_string()])
    );
}

#[tokio::test]
async fn run_repo_overview_returns_index_not_ready_without_repo_entity_publication() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let search_plane_root = temp.path().join("search-plane");
    let studio = StudioState::new_with_bootstrap_ui_config_and_search_plane_root(
        Arc::new(
            bootstrap_builtin_registry()
                .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
        ),
        search_plane_root,
    );
    let repo_root = temp.path().join("PendingRepo");
    fs::create_dir_all(&repo_root).unwrap_or_else(|error| panic!("create pending root: {error}"));
    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: Vec::new(),
        repo_projects: vec![UiRepoProjectConfig {
            id: "pending/repo".to_string(),
            root: Some(repo_root.display().to_string()),
            url: Some("https://github.com/example/PendingRepo".to_string()),
            git_ref: None,
            refresh: None,
            plugins: vec!["julia-code-parser".to_string()],
        }],
    });
    let state = Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(studio),
    });

    let error = match run_repo_overview(Arc::clone(&state), "pending/repo".to_string()).await {
        Ok(result) => {
            panic!("repo overview should require a published repo_entity corpus, got {result:?}")
        }
        Err(error) => error,
    };

    assert_eq!(error.code(), "INDEX_NOT_READY");
    assert_eq!(error.status(), axum::http::StatusCode::CONFLICT);
    assert_eq!(error.error.details.as_deref(), Some("repo_entity"));
}

#[tokio::test]
async fn run_repo_overview_prefers_repo_entity_publication_summary_before_live_analysis() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let search_plane_root = temp.path().join("search-plane");
    let studio = StudioState::new_with_bootstrap_ui_config_and_search_plane_root(
        Arc::new(
            bootstrap_builtin_registry()
                .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
        ),
        search_plane_root,
    );
    let published_root = temp.path().join("PublishedRepo");
    fs::create_dir_all(&published_root)
        .unwrap_or_else(|error| panic!("create published root: {error}"));
    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: Vec::new(),
        repo_projects: vec![UiRepoProjectConfig {
            id: "published/repo".to_string(),
            root: Some(published_root.display().to_string()),
            url: Some("https://github.com/example/PublishedRepo".to_string()),
            git_ref: None,
            refresh: None,
            plugins: vec!["julia-code-parser".to_string()],
        }],
    });
    let analysis = RepositoryAnalysisOutput {
        modules: vec![ModuleRecord {
            repo_id: "published/repo".to_string(),
            module_id: "module:Published".to_string(),
            qualified_name: "Published".to_string(),
            path: "src/Published.jl".to_string(),
        }],
        symbols: vec![SymbolRecord {
            repo_id: "published/repo".to_string(),
            symbol_id: "symbol:solve".to_string(),
            module_id: Some("module:Published".to_string()),
            name: "solve".to_string(),
            qualified_name: "Published.solve".to_string(),
            kind: RepoSymbolKind::Function,
            path: "src/Published.jl".to_string(),
            line_start: Some(3),
            line_end: Some(3),
            signature: Some("solve()".to_string()),
            audit_status: None,
            verification_state: None,
            attributes: BTreeMap::new(),
        }],
        examples: vec![ExampleRecord {
            repo_id: "published/repo".to_string(),
            example_id: "example:solve".to_string(),
            title: "Solve example".to_string(),
            path: "examples/solve.jl".to_string(),
            summary: Some("Shows solve".to_string()),
        }],
        ..RepositoryAnalysisOutput::default()
    };
    let documents = vec![
        RepoCodeDocument {
            path: "src/Published.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from("module Published\nsolve() = nothing\nend\n"),
            size_bytes: 39,
            modified_unix_ms: 1,
        },
        RepoCodeDocument {
            path: "examples/solve.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from("using Published\nsolve()\n"),
            size_bytes: 23,
            modified_unix_ms: 1,
        },
    ];
    studio
        .search_plane
        .publish_repo_entities_with_revision("published/repo", &analysis, &documents, Some("rev-1"))
        .await
        .unwrap_or_else(|error| panic!("publish repo entities: {error}"));
    let state = Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(studio),
    });

    let overview = run_repo_overview(Arc::clone(&state), "published/repo".to_string())
        .await
        .unwrap_or_else(|error| {
            panic!("publication-backed repo overview should succeed: {error:?}")
        });

    assert_eq!(overview.repo_id, "published/repo");
    assert_eq!(overview.display_name, "Published");
    assert_eq!(overview.revision.as_deref(), Some("rev-1"));
    assert_eq!(overview.module_count, 1);
    assert_eq!(overview.symbol_count, 1);
    assert_eq!(overview.example_count, 1);
    assert_eq!(overview.doc_count, 3);
    assert_eq!(
        overview.hierarchical_uri.as_deref(),
        Some("repo://published/repo")
    );
}
