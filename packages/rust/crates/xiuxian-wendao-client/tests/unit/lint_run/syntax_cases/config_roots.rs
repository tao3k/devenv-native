use anyhow::Result;
use tempfile::TempDir;

use super::{common_doc, run_lint};

#[test]
fn lint_uses_wendao_configured_project_roots_when_no_paths_are_given() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("frontend"))?;
    std::fs::write(
        temp.path().join("wendao.toml"),
        "[link_graph.projects.frontend]\nroot = \"frontend\"\n",
    )?;
    std::fs::write(
        temp.path().join("frontend/guide.md"),
        common_doc("Frontend Guide"),
    )?;
    std::fs::write(temp.path().join("loose.md"), "---\ntags: [broken\n---\n")?;

    let (status, stdout) = run_lint(&temp, None)?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Markdown lint passed: checked 1 file(s), 0 issue(s)."),
        "{stdout}"
    );
    Ok(())
}

#[test]
fn lint_skips_managed_remote_project_roots_when_paths_are_omitted() -> Result<()> {
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
    std::fs::write(
        temp.path().join("frontend/guide.md"),
        common_doc("Frontend Guide"),
    )?;
    std::fs::write(
        temp.path().join("readonly-mirror/broken.md"),
        "---\ntags: [broken\n---\n",
    )?;

    let (status, stdout) = run_lint(&temp, None)?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Markdown lint passed: checked 1 file(s), 0 issue(s)."),
        "{stdout}"
    );
    Ok(())
}

#[test]
fn lint_skips_explicit_read_only_project_roots_when_paths_are_omitted() -> Result<()> {
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
    std::fs::write(
        temp.path().join("frontend/guide.md"),
        common_doc("Frontend Guide"),
    )?;
    std::fs::write(
        temp.path().join("readonly-local/broken.md"),
        "---\ntags: [broken\n---\n",
    )?;

    let (status, stdout) = run_lint(&temp, None)?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Markdown lint passed: checked 1 file(s), 0 issue(s)."),
        "{stdout}"
    );
    Ok(())
}

#[test]
fn lint_honors_explicit_read_only_false_for_managed_remote_projects() -> Result<()> {
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
    std::fs::write(
        temp.path().join("frontend/guide.md"),
        common_doc("Frontend Guide"),
    )?;
    std::fs::write(
        temp.path().join("mirror/broken.md"),
        "---\ntags: [broken\n---\n",
    )?;

    let (status, stdout) = run_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("mirror/broken.md"), "{stdout}");
    assert!(stdout.contains("invalid_frontmatter_yaml"), "{stdout}");
    Ok(())
}
