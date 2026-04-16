use std::fs;
use std::path::Path;

use crate::gateway::studio::router::StudioState;
use crate::gateway::studio::types::{UiConfig, UiProjectConfig};
use crate::gateway::studio::vfs::content::{read_content, read_raw_content, resolve_vfs_file_path};

struct NestedFrontendFixture {
    _temp_dir: tempfile::TempDir,
    state: StudioState,
}

fn workspace_local_alias(state: &StudioState, relative_path: &str) -> String {
    let relative_root = state
        .config_root
        .strip_prefix(state.project_root.as_path())
        .unwrap_or_else(|error| panic!("config root should stay under project root: {error}"));
    relative_root
        .join(Path::new(relative_path))
        .to_string_lossy()
        .replace('\\', "/")
}

fn build_state_for_nested_frontend_config() -> NestedFrontendFixture {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let repo_root = temp_dir.path().join("repo");
    let docs_dir = repo_root.join("docs/02_dev");
    let frontend_root = repo_root.join(".data/wendao-frontend");
    let frontend_docs_dir = frontend_root.join("docs");

    fs::create_dir_all(docs_dir.as_path()).unwrap_or_else(|error| panic!("docs dir: {error}"));
    fs::create_dir_all(frontend_root.as_path())
        .unwrap_or_else(|error| panic!("frontend root: {error}"));
    fs::create_dir_all(frontend_docs_dir.as_path())
        .unwrap_or_else(|error| panic!("frontend docs dir: {error}"));
    fs::write(docs_dir.join("HANDBOOK.md"), "# handbook\n")
        .unwrap_or_else(|error| panic!("handbook: {error}"));
    fs::write(docs_dir.join("ARCHITECTURE.pdf"), b"%PDF-1.7\nmultimodal\n")
        .unwrap_or_else(|error| panic!("pdf: {error}"));
    fs::write(
        frontend_docs_dir.join("FRONTEND_GUIDE.md"),
        "# frontend guide\n",
    )
    .unwrap_or_else(|error| panic!("frontend guide: {error}"));

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

    NestedFrontendFixture {
        _temp_dir: temp_dir,
        state,
    }
}

#[test]
fn resolve_vfs_file_path_accepts_canonical_project_scoped_docs_path() {
    let fixture = build_state_for_nested_frontend_config();

    let resolved = resolve_vfs_file_path(&fixture.state, "kernel/docs/02_dev/HANDBOOK.md")
        .unwrap_or_else(|error| panic!("resolve vfs path: {error:?}"));

    assert!(resolved.ends_with("repo/docs/02_dev/HANDBOOK.md"));
}

#[test]
fn resolve_vfs_file_path_rejects_unscoped_docs_alias() {
    let fixture = build_state_for_nested_frontend_config();

    let Err(error) = resolve_vfs_file_path(&fixture.state, "docs/02_dev/HANDBOOK.md") else {
        panic!("unscoped VFS alias should fail");
    };

    assert_eq!(error.error.code, "NOT_FOUND");
}

#[test]
fn resolve_vfs_file_path_accepts_frontend_project_scoped_docs_path() {
    let fixture = build_state_for_nested_frontend_config();

    let resolved = resolve_vfs_file_path(&fixture.state, "frontend/docs/FRONTEND_GUIDE.md")
        .unwrap_or_else(|error| panic!("resolve frontend project-scoped vfs path: {error:?}"));

    assert!(resolved.ends_with("repo/.data/wendao-frontend/docs/FRONTEND_GUIDE.md"));
}

#[tokio::test]
async fn read_content_accepts_frontend_project_scoped_docs_path() {
    let fixture = build_state_for_nested_frontend_config();

    let payload = read_content(&fixture.state, "frontend/docs/FRONTEND_GUIDE.md")
        .await
        .unwrap_or_else(|error| panic!("read frontend project-scoped content: {error:?}"));

    assert_eq!(payload.path, "frontend/docs/FRONTEND_GUIDE.md");
    assert_eq!(payload.content, "# frontend guide\n");
}

#[tokio::test]
async fn read_content_rejects_frontend_workspace_local_alias() {
    let fixture = build_state_for_nested_frontend_config();
    let alias = workspace_local_alias(&fixture.state, "docs/FRONTEND_GUIDE.md");

    let Err(error) = read_content(&fixture.state, alias.as_str()).await else {
        panic!("workspace-local VFS alias should fail");
    };

    assert_eq!(error.error.code, "NOT_FOUND");
}

#[tokio::test]
async fn read_raw_content_preserves_binary_payload_and_inferrs_pdf_content_type() {
    let fixture = build_state_for_nested_frontend_config();

    let payload = read_raw_content(&fixture.state, "kernel/docs/02_dev/ARCHITECTURE.pdf")
        .await
        .unwrap_or_else(|error| panic!("read raw content: {error:?}"));

    assert_eq!(payload.content_type, "application/pdf");
    assert_eq!(payload.content, b"%PDF-1.7\nmultimodal\n");
}
