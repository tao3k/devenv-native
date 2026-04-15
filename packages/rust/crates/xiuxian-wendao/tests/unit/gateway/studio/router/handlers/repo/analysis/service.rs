use std::sync::Arc;

use crate::analyzers::bootstrap_builtin_registry;
use crate::gateway::studio::router::handlers::repo::analysis::service::run_repo_overview;
use crate::gateway::studio::router::{GatewayState, StudioState};
use crate::gateway::studio::types::{UiConfig, UiRepoProjectConfig};

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
