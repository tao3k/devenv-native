use std::path::PathBuf;

use tempfile::TempDir;
use xiuxian_wendao_client::MarkdownLintArgs;

#[path = "../../src/lint/discovery/mod.rs"]
mod discovery;

#[test]
fn skips_default_generated_dirs() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::create_dir_all(temp.path().join("docs")).expect("docs dir should exist");
    std::fs::create_dir_all(temp.path().join("target")).expect("target dir should exist");
    std::fs::write(temp.path().join("docs/guide.md"), "# Guide\n").expect("guide should exist");
    std::fs::write(temp.path().join("target/generated.md"), "# Generated\n")
        .expect("generated should exist");

    let files = discovery::collect_markdown_files(
        temp.path(),
        &MarkdownLintArgs {
            paths: Vec::new(),
            skip_dirs: Vec::new(),
        },
    )
    .expect("collection should succeed");

    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("docs/guide.md"));
}

#[test]
fn keeps_relative_display_paths() {
    let root = PathBuf::from("/tmp/demo");
    let path = PathBuf::from("/tmp/demo/docs/guide.md");
    assert_eq!(
        discovery::display_path(path.as_path(), root.as_path()),
        "docs/guide.md"
    );
}

#[test]
fn uses_configured_project_roots_when_paths_are_omitted() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::create_dir_all(temp.path().join("frontend")).expect("frontend dir should exist");
    std::fs::create_dir_all(temp.path().join("backend")).expect("backend dir should exist");
    std::fs::write(
        temp.path().join("wendao.toml"),
        concat!(
            "[link_graph.projects.frontend]\n",
            "root = \"frontend\"\n\n",
            "[link_graph.projects.backend]\n",
            "root = \"backend\"\n",
        ),
    )
    .expect("config should exist");
    std::fs::write(temp.path().join("frontend/guide.md"), "# Frontend\n")
        .expect("frontend guide should exist");
    std::fs::write(temp.path().join("backend/guide.md"), "# Backend\n")
        .expect("backend guide should exist");
    std::fs::write(temp.path().join("loose.md"), "# Loose\n").expect("loose file should exist");

    let files = discovery::collect_markdown_files(
        temp.path(),
        &MarkdownLintArgs {
            paths: Vec::new(),
            skip_dirs: Vec::new(),
        },
    )
    .expect("collection should succeed");

    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|path| path.ends_with("frontend/guide.md")));
    assert!(files.iter().any(|path| path.ends_with("backend/guide.md")));
    assert!(!files.iter().any(|path| path.ends_with("loose.md")));
}

#[test]
fn explicit_paths_override_configured_project_roots() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::create_dir_all(temp.path().join("frontend")).expect("frontend dir should exist");
    std::fs::create_dir_all(temp.path().join("backend")).expect("backend dir should exist");
    std::fs::write(
        temp.path().join("wendao.toml"),
        concat!(
            "[link_graph.projects.frontend]\n",
            "root = \"frontend\"\n\n",
            "[link_graph.projects.backend]\n",
            "root = \"backend\"\n",
        ),
    )
    .expect("config should exist");
    std::fs::write(temp.path().join("frontend/guide.md"), "# Frontend\n")
        .expect("frontend guide should exist");
    std::fs::write(temp.path().join("backend/guide.md"), "# Backend\n")
        .expect("backend guide should exist");

    let files = discovery::collect_markdown_files(
        temp.path(),
        &MarkdownLintArgs {
            paths: vec![PathBuf::from("frontend")],
            skip_dirs: Vec::new(),
        },
    )
    .expect("collection should succeed");

    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("frontend/guide.md"));
}

#[test]
fn omits_managed_remote_projects_from_default_configured_roots() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::create_dir_all(temp.path().join("frontend")).expect("frontend dir should exist");
    std::fs::create_dir_all(temp.path().join("readonly-mirror"))
        .expect("readonly mirror dir should exist");
    std::fs::write(
        temp.path().join("wendao.toml"),
        concat!(
            "[link_graph.projects.frontend]\n",
            "root = \"frontend\"\n\n",
            "[link_graph.projects.readonly]\n",
            "root = \"readonly-mirror\"\n",
            "url = \"https://example.com/repo.git\"\n",
        ),
    )
    .expect("config should exist");
    std::fs::write(temp.path().join("frontend/guide.md"), "# Frontend\n")
        .expect("frontend guide should exist");
    std::fs::write(temp.path().join("readonly-mirror/guide.md"), "# Readonly\n")
        .expect("readonly guide should exist");

    let files = discovery::collect_markdown_files(
        temp.path(),
        &MarkdownLintArgs {
            paths: Vec::new(),
            skip_dirs: Vec::new(),
        },
    )
    .expect("collection should succeed");

    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("frontend/guide.md"));
}

#[test]
fn omits_explicit_read_only_projects_from_default_configured_roots() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::create_dir_all(temp.path().join("frontend")).expect("frontend dir should exist");
    std::fs::create_dir_all(temp.path().join("readonly-local"))
        .expect("readonly local dir should exist");
    std::fs::write(
        temp.path().join("wendao.toml"),
        concat!(
            "[link_graph.projects.frontend]\n",
            "root = \"frontend\"\n\n",
            "[link_graph.projects.readonly]\n",
            "root = \"readonly-local\"\n",
            "read_only = true\n",
        ),
    )
    .expect("config should exist");
    std::fs::write(temp.path().join("frontend/guide.md"), "# Frontend\n")
        .expect("frontend guide should exist");
    std::fs::write(temp.path().join("readonly-local/guide.md"), "# Readonly\n")
        .expect("readonly guide should exist");

    let files = discovery::collect_markdown_files(
        temp.path(),
        &MarkdownLintArgs {
            paths: Vec::new(),
            skip_dirs: Vec::new(),
        },
    )
    .expect("collection should succeed");

    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("frontend/guide.md"));
}

#[test]
fn explicit_read_only_false_overrides_managed_remote_inference() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::create_dir_all(temp.path().join("frontend")).expect("frontend dir should exist");
    std::fs::create_dir_all(temp.path().join("mirror")).expect("mirror dir should exist");
    std::fs::write(
        temp.path().join("wendao.toml"),
        concat!(
            "[link_graph.projects.frontend]\n",
            "root = \"frontend\"\n\n",
            "[link_graph.projects.mirror]\n",
            "root = \"mirror\"\n",
            "url = \"https://example.com/repo.git\"\n",
            "read_only = false\n",
        ),
    )
    .expect("config should exist");
    std::fs::write(temp.path().join("frontend/guide.md"), "# Frontend\n")
        .expect("frontend guide should exist");
    std::fs::write(temp.path().join("mirror/guide.md"), "# Mirror\n")
        .expect("mirror guide should exist");

    let files = discovery::collect_markdown_files(
        temp.path(),
        &MarkdownLintArgs {
            paths: Vec::new(),
            skip_dirs: Vec::new(),
        },
    )
    .expect("collection should succeed");

    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|path| path.ends_with("frontend/guide.md")));
    assert!(files.iter().any(|path| path.ends_with("mirror/guide.md")));
}
