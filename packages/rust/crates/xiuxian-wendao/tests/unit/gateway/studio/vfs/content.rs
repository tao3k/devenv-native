use std::fs;

use crate::gateway::studio::router::StudioState;
use crate::gateway::studio::types::{UiConfig, UiProjectConfig};
use crate::gateway::studio::vfs::content::{read_raw_content, resolve_vfs_file_path};

fn build_state_for_nested_frontend_config() -> (tempfile::TempDir, StudioState) {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let repo_root = temp_dir.path().join("repo");
    let docs_dir = repo_root.join("docs/02_dev");
    let frontend_root = repo_root.join(".data/wendao-frontend");

    fs::create_dir_all(docs_dir.as_path()).unwrap_or_else(|error| panic!("docs dir: {error}"));
    fs::create_dir_all(frontend_root.as_path())
        .unwrap_or_else(|error| panic!("frontend root: {error}"));
    fs::write(docs_dir.join("HANDBOOK.md"), "# handbook\n")
        .unwrap_or_else(|error| panic!("handbook: {error}"));
    fs::write(docs_dir.join("ARCHITECTURE.pdf"), b"%PDF-1.7\nmultimodal\n")
        .unwrap_or_else(|error| panic!("pdf: {error}"));

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

    (temp_dir, state)
}

#[test]
fn resolve_vfs_file_path_accepts_unscoped_docs_path_via_studio_display_path() {
    let (_temp_dir, state) = build_state_for_nested_frontend_config();

    let resolved = resolve_vfs_file_path(&state, "docs/02_dev/HANDBOOK.md")
        .unwrap_or_else(|error| panic!("resolve vfs path: {error:?}"));

    assert!(resolved.ends_with("repo/docs/02_dev/HANDBOOK.md"));
}

#[test]
fn resolve_vfs_file_path_keeps_explicit_project_prefix_working() {
    let (_temp_dir, state) = build_state_for_nested_frontend_config();

    let resolved = resolve_vfs_file_path(&state, "kernel/docs/02_dev/HANDBOOK.md")
        .unwrap_or_else(|error| panic!("resolve vfs path: {error:?}"));

    assert!(resolved.ends_with("repo/docs/02_dev/HANDBOOK.md"));
}

#[tokio::test]
async fn read_raw_content_preserves_binary_payload_and_inferrs_pdf_content_type() {
    let (_temp_dir, state) = build_state_for_nested_frontend_config();

    let payload = read_raw_content(&state, "docs/02_dev/ARCHITECTURE.pdf")
        .await
        .unwrap_or_else(|error| panic!("read raw content: {error:?}"));

    assert_eq!(payload.content_type, "application/pdf");
    assert_eq!(payload.content, b"%PDF-1.7\nmultimodal\n");
}
