use crate::gateway::studio::StudioState;
use crate::gateway::studio::types::{StudioNavigationTarget, UiConfig, UiProjectConfig};
use crate::gateway::studio::vfs::flight::{
    build_vfs_resolve_response, vfs_navigation_target_batch,
};

fn workspace_local_alias(state: &StudioState, relative_path: &str) -> String {
    state
        .config_root
        .strip_prefix(state.project_root.as_path())
        .unwrap_or_else(|error| panic!("config root should stay under project root: {error}"))
        .join(relative_path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn build_state_for_nested_frontend_resolve() -> StudioState {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let repo_root = temp_dir.path().join("repo");
    let frontend_root = repo_root.join(".data/wendao-frontend");

    std::fs::create_dir_all(frontend_root.join("docs"))
        .unwrap_or_else(|error| panic!("frontend docs dir: {error}"));

    let mut state = StudioState::new();
    state.project_root = repo_root;
    state.config_root = frontend_root;
    state.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![
            UiProjectConfig {
                name: "kernel".to_string(),
                root: "../..".to_string(),
                dirs: vec!["docs".to_string()],
            },
            UiProjectConfig {
                name: "frontend".to_string(),
                root: ".".to_string(),
                dirs: vec!["docs".to_string()],
            },
        ],
        repo_projects: Vec::new(),
    });
    state
}

#[tokio::test]
async fn build_vfs_resolve_response_prefixes_project_for_relative_docs_path() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let mut state = StudioState::new();
    state.project_root = temp_dir.path().to_path_buf();
    state.config_root = temp_dir.path().to_path_buf();
    state.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![UiProjectConfig {
            name: "main".to_string(),
            root: ".".to_string(),
            dirs: vec!["docs".to_string()],
        }],
        repo_projects: Vec::new(),
    });

    let target = build_vfs_resolve_response(&state, "docs/index.md")
        .unwrap_or_else(|error| panic!("build VFS resolve response: {error:?}"));

    assert_eq!(target.path, "main/docs/index.md");
    assert_eq!(target.project_name.as_deref(), Some("main"));
}

#[tokio::test]
async fn build_vfs_resolve_response_canonicalizes_frontend_workspace_local_docs_path() {
    let state = build_state_for_nested_frontend_resolve();
    let alias = workspace_local_alias(&state, "docs/FRONTEND_GUIDE.md");

    let target = build_vfs_resolve_response(&state, alias.as_str())
        .unwrap_or_else(|error| panic!("build nested frontend VFS resolve response: {error:?}"));

    assert_eq!(target.path, "frontend/docs/FRONTEND_GUIDE.md");
    assert_eq!(target.project_name.as_deref(), Some("frontend"));
}

#[tokio::test]
async fn build_vfs_resolve_response_rejects_blank_path() {
    let Err(error) = build_vfs_resolve_response(&StudioState::new(), "   ") else {
        panic!("blank path should fail");
    };
    assert_eq!(error.error.code, "MISSING_PATH");
}

#[test]
fn vfs_navigation_target_batch_preserves_project_metadata() {
    let batch = vfs_navigation_target_batch(&StudioNavigationTarget {
        path: "kernel/docs/index.md".to_string(),
        category: "file".to_string(),
        project_name: Some("kernel".to_string()),
        root_label: Some("project".to_string()),
        line: Some(7),
        line_end: Some(9),
        column: Some(3),
    })
    .unwrap_or_else(|error| panic!("build VFS navigation batch: {error}"));
    assert_eq!(batch.num_rows(), 1);
    assert_eq!(batch.num_columns(), 7);
}
