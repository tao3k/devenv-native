use anyhow::Result;
use std::path::PathBuf;

use tempfile::TempDir;
use xiuxian_wendao_client::MarkdownLintArgs;

#[path = "../../src/lint/discovery/mod.rs"]
mod discovery;

#[test]
fn skips_default_transient_and_generated_dirs() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("docs"))?;
    std::fs::create_dir_all(temp.path().join(".cache"))?;
    std::fs::create_dir_all(temp.path().join(".data"))?;
    std::fs::create_dir_all(temp.path().join(".run"))?;
    std::fs::create_dir_all(temp.path().join(".config"))?;
    std::fs::create_dir_all(temp.path().join(".bin"))?;
    std::fs::create_dir_all(temp.path().join("node_modules"))?;
    std::fs::create_dir_all(temp.path().join("target"))?;
    std::fs::write(temp.path().join("docs/guide.md"), "# Guide\n")?;
    std::fs::write(temp.path().join(".cache/generated.md"), "# Generated\n")?;
    std::fs::write(temp.path().join(".data/generated.md"), "# Generated\n")?;
    std::fs::write(temp.path().join(".run/generated.md"), "# Generated\n")?;
    std::fs::write(temp.path().join(".config/generated.md"), "# Generated\n")?;
    std::fs::write(temp.path().join(".bin/generated.md"), "# Generated\n")?;
    std::fs::write(
        temp.path().join("node_modules/generated.md"),
        "# Generated\n",
    )?;
    std::fs::write(temp.path().join("target/generated.md"), "# Generated\n")?;

    let files = discovery::collect_markdown_files(
        temp.path(),
        &MarkdownLintArgs {
            paths: Vec::new(),
            skip_dirs: Vec::new(),
        },
    )?;

    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("docs/guide.md"));
    Ok(())
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
fn identifies_first_transient_repo_dir_in_relative_paths() {
    let path = PathBuf::from("docs/.data/generated/note.md");
    assert_eq!(
        discovery::first_transient_repo_dir(path.as_path()),
        Some(".data")
    );
}

#[test]
fn uses_configured_project_roots_when_paths_are_omitted() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("frontend"))?;
    std::fs::create_dir_all(temp.path().join("backend"))?;
    std::fs::write(
        temp.path().join("wendao.toml"),
        concat!(
            "[link_graph.projects.frontend]\n",
            "root = \"frontend\"\n\n",
            "[link_graph.projects.backend]\n",
            "root = \"backend\"\n",
        ),
    )?;
    std::fs::write(temp.path().join("frontend/guide.md"), "# Frontend\n")?;
    std::fs::write(temp.path().join("backend/guide.md"), "# Backend\n")?;
    std::fs::write(temp.path().join("loose.md"), "# Loose\n")?;

    let files = discovery::collect_markdown_files(
        temp.path(),
        &MarkdownLintArgs {
            paths: Vec::new(),
            skip_dirs: Vec::new(),
        },
    )?;

    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|path| path.ends_with("frontend/guide.md")));
    assert!(files.iter().any(|path| path.ends_with("backend/guide.md")));
    assert!(!files.iter().any(|path| path.ends_with("loose.md")));
    Ok(())
}

#[test]
fn explicit_paths_override_configured_project_roots() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("frontend"))?;
    std::fs::create_dir_all(temp.path().join("backend"))?;
    std::fs::write(
        temp.path().join("wendao.toml"),
        concat!(
            "[link_graph.projects.frontend]\n",
            "root = \"frontend\"\n\n",
            "[link_graph.projects.backend]\n",
            "root = \"backend\"\n",
        ),
    )?;
    std::fs::write(temp.path().join("frontend/guide.md"), "# Frontend\n")?;
    std::fs::write(temp.path().join("backend/guide.md"), "# Backend\n")?;

    let files = discovery::collect_markdown_files(
        temp.path(),
        &MarkdownLintArgs {
            paths: vec![PathBuf::from("frontend")],
            skip_dirs: Vec::new(),
        },
    )?;

    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("frontend/guide.md"));
    Ok(())
}

#[test]
fn omits_managed_remote_projects_from_default_configured_roots() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("frontend"))?;
    std::fs::create_dir_all(temp.path().join("readonly-mirror"))?;
    std::fs::write(
        temp.path().join("wendao.toml"),
        concat!(
            "[link_graph.projects.frontend]\n",
            "root = \"frontend\"\n\n",
            "[link_graph.projects.readonly]\n",
            "root = \"readonly-mirror\"\n",
            "url = \"https://example.com/repo.git\"\n",
        ),
    )?;
    std::fs::write(temp.path().join("frontend/guide.md"), "# Frontend\n")?;
    std::fs::write(temp.path().join("readonly-mirror/guide.md"), "# Readonly\n")?;

    let files = discovery::collect_markdown_files(
        temp.path(),
        &MarkdownLintArgs {
            paths: Vec::new(),
            skip_dirs: Vec::new(),
        },
    )?;

    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("frontend/guide.md"));
    Ok(())
}

#[test]
fn omits_explicit_read_only_projects_from_default_configured_roots() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("frontend"))?;
    std::fs::create_dir_all(temp.path().join("readonly-local"))?;
    std::fs::write(
        temp.path().join("wendao.toml"),
        concat!(
            "[link_graph.projects.frontend]\n",
            "root = \"frontend\"\n\n",
            "[link_graph.projects.readonly]\n",
            "root = \"readonly-local\"\n",
            "read_only = true\n",
        ),
    )?;
    std::fs::write(temp.path().join("frontend/guide.md"), "# Frontend\n")?;
    std::fs::write(temp.path().join("readonly-local/guide.md"), "# Readonly\n")?;

    let files = discovery::collect_markdown_files(
        temp.path(),
        &MarkdownLintArgs {
            paths: Vec::new(),
            skip_dirs: Vec::new(),
        },
    )?;

    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("frontend/guide.md"));
    Ok(())
}

#[test]
fn explicit_read_only_false_overrides_managed_remote_inference() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("frontend"))?;
    std::fs::create_dir_all(temp.path().join("mirror"))?;
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
    )?;
    std::fs::write(temp.path().join("frontend/guide.md"), "# Frontend\n")?;
    std::fs::write(temp.path().join("mirror/guide.md"), "# Mirror\n")?;

    let files = discovery::collect_markdown_files(
        temp.path(),
        &MarkdownLintArgs {
            paths: Vec::new(),
            skip_dirs: Vec::new(),
        },
    )?;

    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|path| path.ends_with("frontend/guide.md")));
    assert!(files.iter().any(|path| path.ends_with("mirror/guide.md")));
    Ok(())
}
