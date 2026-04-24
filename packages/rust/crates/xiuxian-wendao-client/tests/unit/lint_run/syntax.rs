use anyhow::Result;
use tempfile::TempDir;

use super::run_markdown_lint;

#[test]
fn lint_reports_invalid_yaml_frontmatter() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::write(temp.path().join("demo.md"), "---\ntags: [demo\n---\n")?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

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

    let (status, stdout) = run_markdown_lint(&temp, None)?;

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

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("missing_frontmatter_title"));
    assert!(stdout.contains("kind: repo_authoring_policy"));
    assert!(
        stdout.contains("problem: Ordinary document frontmatter must include a non-empty `title`.")
    );
    Ok(())
}

#[test]
fn lint_reports_missing_skill_frontmatter_name() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("skills/demo"))?;
    std::fs::write(
        temp.path().join("skills/demo/SKILL.md"),
        "---\nmetadata:\n  version: \"1.0.0\"\n---\n# Demo Skill\n",
    )?;

    let (status, stdout) = run_markdown_lint(&temp, Some("skills"))?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("missing_skill_frontmatter_name"));
    assert!(stdout.contains("kind: repo_authoring_policy"));
    assert!(stdout.contains(
        "problem: Skill-shaped document frontmatter must include a non-empty top-level `name`."
    ));
    Ok(())
}

#[test]
fn lint_reports_missing_skill_frontmatter_metadata() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("skills/demo"))?;
    std::fs::write(
        temp.path().join("skills/demo/SKILL.md"),
        "---\nname: demo-skill\n---\n# Demo Skill\n",
    )?;

    let (status, stdout) = run_markdown_lint(&temp, Some("skills"))?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("missing_skill_frontmatter_metadata"));
    assert!(stdout.contains("kind: repo_authoring_policy"));
    assert!(stdout.contains(
        "problem: Skill-shaped document frontmatter must contain a top-level `metadata` mapping."
    ));
    Ok(())
}

#[test]
fn lint_accepts_skill_md_with_strict_skill_frontmatter() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("skills/demo"))?;
    std::fs::write(
        temp.path().join("skills/demo/SKILL.md"),
        "---\nname: demo-skill\nmetadata:\n  version: \"1.0.0\"\n---\n# Demo Skill\n",
    )?;

    let (status, stdout) = run_markdown_lint(&temp, Some("skills"))?;

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains("Markdown lint passed: checked 1 file(s), 0 issue(s)."),
        "{stdout}"
    );
    Ok(())
}

#[test]
fn lint_accepts_kind_marked_skill_doc_with_strict_skill_frontmatter() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::write(
        temp.path().join("planner.md"),
        "---\nkind: SKILL.md\nname: planner\nmetadata:\n  version: \"1.0.0\"\n---\n# Planner\n",
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains("Markdown lint passed: checked 1 file(s), 0 issue(s)."),
        "{stdout}"
    );
    Ok(())
}

#[test]
fn lint_reports_unclosed_frontmatter() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::write(temp.path().join("demo.md"), "---\ntitle: demo\nbody\n")?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("unclosed_frontmatter"));
    assert!(stdout.contains("kind: official_syntax"));
    assert!(stdout.contains("problem: YAML frontmatter opens but never closes."));
    assert!(stdout.contains(
        "expected: Close the frontmatter with `---` or `...` before the document body begins."
    ));
    Ok(())
}

#[test]
fn lint_reports_invalid_utf8_as_official_syntax() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::write(temp.path().join("demo.md"), vec![0xff, 0xfe, 0xfd])?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("invalid_utf8"));
    assert!(stdout.contains("kind: official_syntax"));
    assert!(stdout.contains("problem: Markdown file is not valid UTF-8."));
    Ok(())
}

#[test]
fn lint_reports_unclosed_fence() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::write(
        temp.path().join("demo.md"),
        "---\ntitle: Demo\n---\n# Demo\n```rust\nfn main() {}\n",
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("unclosed_fence"));
    assert!(stdout.contains("kind: official_syntax"));
    assert!(stdout.contains("problem: Fenced code block opens but never closes."));
    assert!(stdout.contains(
        "expected: Add a closing fence with the same marker type and at least the same width."
    ));
    Ok(())
}

#[test]
fn lint_succeeds_for_clean_markdown() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::write(
        temp.path().join("guide.md"),
        "---\ntitle: Demo\n---\n# Heading\n```rust\nfn main() {}\n```\n",
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains("Markdown lint passed: checked 1 file(s), 0 issue(s)."),
        "{stdout}"
    );
    Ok(())
}

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
        "---\ntitle: Frontend Guide\n---\n# Frontend Guide\n",
    )?;
    std::fs::write(temp.path().join("loose.md"), "---\ntags: [broken\n---\n")?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(0));
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
        "---\ntitle: Frontend Guide\n---\n# Frontend Guide\n",
    )?;
    std::fs::write(
        temp.path().join("readonly-mirror/broken.md"),
        "---\ntags: [broken\n---\n",
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(0));
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
        "---\ntitle: Frontend Guide\n---\n# Frontend Guide\n",
    )?;
    std::fs::write(
        temp.path().join("readonly-local/broken.md"),
        "---\ntags: [broken\n---\n",
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(0));
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
        "---\ntitle: Frontend Guide\n---\n# Frontend Guide\n",
    )?;
    std::fs::write(
        temp.path().join("mirror/broken.md"),
        "---\ntags: [broken\n---\n",
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("mirror/broken.md"), "{stdout}");
    assert!(stdout.contains("invalid_frontmatter_yaml"), "{stdout}");
    Ok(())
}
