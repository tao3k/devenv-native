use anyhow::Result;
use tempfile::TempDir;

use super::run_lint;

#[test]
fn lint_reports_invalid_yaml_frontmatter() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::write(temp.path().join("demo.md"), "---\ntags: [demo\n---\n")?;

    let (status, stdout) = run_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("invalid_frontmatter_yaml"));
    assert!(stdout.contains("kind: official_syntax"));
    assert!(stdout.contains("problem: YAML frontmatter is syntactically invalid."));
    assert!(stdout.contains("demo.md"));
    Ok(())
}

#[test]
fn lint_reports_missing_frontmatter() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::write(temp.path().join("demo.md"), "# Heading\nbody\n")?;

    let (status, stdout) = run_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("missing_frontmatter"));
    assert!(stdout.contains("kind: repo_authoring_policy"));
    assert!(stdout.contains("problem: Document-level YAML frontmatter is required."));
    Ok(())
}

#[test]
fn lint_reports_missing_frontmatter_title() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::write(
        temp.path().join("demo.md"),
        "---\ntags:\n  - demo\n---\n# Heading\n",
    )?;

    let (status, stdout) = run_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("missing_frontmatter_title"));
    assert!(stdout.contains("kind: repo_authoring_policy"));
    assert!(
        stdout.contains("problem: Ordinary document frontmatter must include a non-empty `title`.")
    );
    Ok(())
}
