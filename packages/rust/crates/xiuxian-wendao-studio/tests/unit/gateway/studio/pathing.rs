use std::fs;
use std::path::Path;

use uuid::Uuid;

use crate::studio::pathing::{resolve_path_like, studio_display_path};
use crate::studio::test_support::commit_all;
use crate::studio::{StudioState, configured_repositories};
use xiuxian_git_repo::SyncMode;
use xiuxian_wendao::analyzers::resolve_registered_repository_source;
use xiuxian_wendao::search::contracts::{UiConfig, UiProjectConfig, UiRepoProjectConfig};

fn init_git_repository(root: &Path) {
    crate::studio::test_support::init_git_repository(root);
    fs::create_dir_all(root.join("src")).unwrap_or_else(|error| panic!("create src dir: {error}"));
    fs::write(root.join("src").join("lib.rs"), "pub fn kernel() {}\n")
        .unwrap_or_else(|error| panic!("write rust source: {error}"));
    commit_all(root, "init");
}

#[test]
fn resolve_path_like_expands_tilde_prefixed_home_paths() {
    let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) else {
        return;
    };

    let resolved = resolve_path_like(Path::new("/tmp/studio"), "~/workspace/docs")
        .unwrap_or_else(|| panic!("tilde-prefixed path should resolve"));

    assert_eq!(resolved, home.join("workspace/docs"));
}

#[test]
fn resolve_path_like_keeps_relative_paths_rooted_at_base() {
    let resolved = resolve_path_like(Path::new("/tmp/studio"), "docs")
        .unwrap_or_else(|| panic!("relative path should resolve"));

    assert_eq!(resolved, std::path::PathBuf::from("/tmp/studio/docs"));
}

#[test]
fn resolve_path_like_normalizes_current_dir_segments() {
    let resolved = resolve_path_like(Path::new("/tmp/studio"), ".")
        .unwrap_or_else(|| panic!("current-dir path should resolve"));

    assert_eq!(resolved, std::path::PathBuf::from("/tmp/studio"));
}

#[test]
fn studio_display_path_prefixes_configured_project_for_relative_paths() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let mut state = StudioState::new();
    state.project_root = temp_dir.path().to_path_buf();
    state.config_root = temp_dir.path().to_path_buf();
    state.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![UiProjectConfig {
            name: "main".to_string(),
            root: ".".to_string(),
            dirs: vec!["docs".to_string(), "skills".to_string()],
        }],
        repo_projects: Vec::new(),
    });

    assert_eq!(
        studio_display_path(&state, "docs/index.md"),
        "main/docs/index.md"
    );
}

#[test]
fn studio_display_path_keeps_existing_project_prefixes() {
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

    assert_eq!(
        studio_display_path(&state, "main/docs/index.md"),
        "main/docs/index.md"
    );
}

#[test]
fn studio_display_path_strips_relative_project_root_prefixes() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let mut state = StudioState::new();
    state.project_root = temp_dir.path().to_path_buf();
    state.config_root = temp_dir.path().to_path_buf();
    state.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: "frontend".to_string(),
            dirs: vec!["docs".to_string()],
        }],
        repo_projects: Vec::new(),
    });

    assert_eq!(
        studio_display_path(&state, "frontend/docs/index.md"),
        "kernel/docs/index.md"
    );
}

#[test]
fn studio_display_path_prefers_project_root_relative_prefix_for_kernel_docs() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let mut state = StudioState::new();
    state.project_root = temp_dir.path().to_path_buf();
    state.config_root = temp_dir.path().join(".data/wendao-frontend");
    state.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![
            UiProjectConfig {
                name: "kernel".to_string(),
                root: ".".to_string(),
                dirs: vec!["docs".to_string()],
            },
            UiProjectConfig {
                name: "main".to_string(),
                root: temp_dir.path().to_path_buf().to_string_lossy().to_string(),
                dirs: vec!["docs".to_string(), "skills".to_string()],
            },
        ],
        repo_projects: Vec::new(),
    });

    assert_eq!(
        studio_display_path(&state, ".data/wendao-frontend/docs/index.md"),
        "kernel/docs/index.md"
    );
    assert_eq!(
        studio_display_path(&state, "docs/index.md"),
        "main/docs/index.md"
    );
}

#[test]
fn studio_display_path_prefixes_repo_project_id_for_managed_checkout_paths() {
    let source = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(source.path());
    let repo_id = format!("repo-pathing-{}", Uuid::new_v4());
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
    .unwrap_or_else(|error| panic!("materialize checkout for pathing: {error}"));
    let checkout_path = materialized.checkout_root.join("src").join("lib.rs");

    assert_eq!(
        studio_display_path(&state, checkout_path.to_string_lossy().as_ref()),
        format!("{repo_id}/src/lib.rs")
    );

    fs::remove_dir_all(materialized.checkout_root)
        .unwrap_or_else(|error| panic!("cleanup managed checkout: {error}"));
    if let Some(mirror_root) = materialized.mirror_root {
        fs::remove_dir_all(mirror_root).ok();
    }
}
