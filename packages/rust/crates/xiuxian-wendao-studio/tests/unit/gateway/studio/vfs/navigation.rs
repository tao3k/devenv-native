use std::fs;
use std::path::Path;

use uuid::Uuid;

use crate::contracts::{UiConfig, UiProjectConfig, UiRepoProjectConfig};
use crate::studio::test_support::commit_all;
use crate::studio::vfs::navigation::resolve_navigation_target;
use crate::studio::{StudioState, configured_repositories};
use xiuxian_git_repo::SyncMode;
use xiuxian_wendao::analyzers::resolve_registered_repository_source;

fn init_git_repository(root: &Path) {
    crate::studio::test_support::init_git_repository(root);
    fs::create_dir_all(root.join("src")).unwrap_or_else(|error| panic!("create src dir: {error}"));
    fs::write(root.join("src").join("lib.rs"), "pub fn lance_nav() {}\n")
        .unwrap_or_else(|error| panic!("write rust source: {error}"));
    commit_all(root, "init");
}

#[test]
fn resolve_navigation_target_prefixes_configured_project_for_relative_docs_path() {
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

    let target = resolve_navigation_target(&state, "docs/index.md");

    assert_eq!(target.path, "main/docs/index.md");
    assert_eq!(target.project_name.as_deref(), Some("main"));
}

#[test]
fn resolve_navigation_target_sets_repo_project_name_for_managed_checkout_paths() {
    let source = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(source.path());
    let repo_id = format!("repo-nav-{}", Uuid::new_v4());
    let state = StudioState::new();
    state.seed_eager_configured_owners_for_tests(UiConfig {
        projects: Vec::new(),
        repo_projects: vec![UiRepoProjectConfig {
            id: repo_id.clone(),
            root: None,
            url: Some(source.path().display().to_string()),
            git_ref: None,
            refresh: Some("manual".to_string()),
            plugins: vec!["ast-grep".to_string()],
        }],
    });
    let repositories = configured_repositories(&state);
    let repository = repositories
        .first()
        .unwrap_or_else(|| panic!("configured repository"));
    let materialized = resolve_registered_repository_source(
        repository,
        state.config_root.as_path(),
        SyncMode::Ensure,
    )
    .unwrap_or_else(|error| panic!("materialize checkout for navigation: {error}"));
    let checkout_path = materialized.checkout_root.join("src").join("lib.rs");

    let target = resolve_navigation_target(&state, checkout_path.to_string_lossy().as_ref());

    assert_eq!(target.path, format!("{repo_id}/src/lib.rs"));
    assert_eq!(target.project_name.as_deref(), Some(repo_id.as_str()));

    fs::remove_dir_all(materialized.checkout_root)
        .unwrap_or_else(|error| panic!("cleanup managed checkout: {error}"));
    if let Some(mirror_root) = materialized.mirror_root {
        fs::remove_dir_all(mirror_root).ok();
    }
}
