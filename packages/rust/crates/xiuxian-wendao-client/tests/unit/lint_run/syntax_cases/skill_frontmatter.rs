use anyhow::Result;
use tempfile::TempDir;

use super::{run_lint, strict_skill_doc};

#[test]
fn lint_reports_missing_skill_frontmatter_name() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("skills/demo"))?;
    std::fs::write(
        temp.path().join("skills/demo/SKILL.md"),
        concat!(
            "---\n",
            "kind: SKILL.md\n",
            "title: Demo Skill\n",
            "category: skills\n",
            "tags:\n",
            "  - demo\n",
            "description: Demo skill\n",
            "author: xiuxian-artisan-workshop\n",
            "date: \"2026-04-26T09:30-07:00\"\n",
            "type: skill\n",
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "  author: xiuxian-artisan-workshop\n",
            "  version: \"1.0.0\"\n",
            "  source: \"https://example.test/skills/demo\"\n",
            "  routing_keywords:\n",
            "    - demo\n",
            "---\n",
            "# Demo Skill\n",
        ),
    )?;

    let (status, stdout) = run_lint(&temp, Some("skills"))?;

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
        concat!(
            "---\n",
            "kind: SKILL.md\n",
            "title: Demo Skill\n",
            "category: skills\n",
            "tags:\n",
            "  - demo\n",
            "description: Demo skill\n",
            "author: xiuxian-artisan-workshop\n",
            "date: \"2026-04-26T09:30-07:00\"\n",
            "type: skill\n",
            "name: demo-skill\n",
            "---\n",
            "# Demo Skill\n",
        ),
    )?;

    let (status, stdout) = run_lint(&temp, Some("skills"))?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("missing_skill_frontmatter_metadata"));
    assert!(stdout.contains("kind: repo_authoring_policy"));
    assert!(stdout.contains(
        "problem: Skill-shaped document frontmatter must contain a top-level `metadata` mapping."
    ));
    Ok(())
}

#[test]
fn lint_reports_invalid_skill_frontmatter_schema() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("skills/demo"))?;
    std::fs::write(
        temp.path().join("skills/demo/SKILL.md"),
        concat!(
            "---\n",
            "kind: SKILL.md\n",
            "title: Demo Skill\n",
            "category: skills\n",
            "tags:\n",
            "  - demo\n",
            "description: Demo skill\n",
            "author: xiuxian-artisan-workshop\n",
            "date: \"2026-04-26T09:30-07:00\"\n",
            "name: demo-skill\n",
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "  version: \"1.0.0\"\n",
            "---\n",
            "# Demo Skill\n",
        ),
    )?;

    let (status, stdout) = run_lint(&temp, Some("skills"))?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("invalid_skill_frontmatter_schema"));
    assert!(stdout.contains("kind: repo_authoring_policy"));
    assert!(stdout.contains(
        "problem: Skill-shaped document frontmatter must satisfy the parser-owned SKILL.md schema."
    ));
    assert!(stdout.contains("detail: skill frontmatter top-level `type` must be `skill`"));
    assert!(stdout.contains("metadata.source"));
    assert!(stdout.contains("metadata.routing_keywords"));
    assert!(stdout.contains(
        "expected: Start from common frontmatter, then add top-level `type: skill`, `name`, `metadata.version`, `metadata.source`, and a non-empty `metadata.routing_keywords` array."
    ));
    Ok(())
}

#[test]
fn lint_reports_invalid_skill_frontmatter_field_type() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("skills/demo"))?;
    std::fs::write(
        temp.path().join("skills/demo/SKILL.md"),
        concat!(
            "---\n",
            "kind: SKILL.md\n",
            "type: skill\n",
            "title: Demo Skill\n",
            "category: skills\n",
            "tags:\n",
            "  - demo\n",
            "author: xiuxian-artisan-workshop\n",
            "date: \"2026-04-26T09:30-07:00\"\n",
            "name: demo-skill\n",
            "description: Demo skill\n",
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "  author: xiuxian-artisan-workshop\n",
            "  version: \"1.0.0\"\n",
            "  source: \"https://example.test/skills/demo\"\n",
            "  routing_keywords: demo\n",
            "---\n",
            "# Demo Skill\n",
        ),
    )?;

    let (status, stdout) = run_lint(&temp, Some("skills"))?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("invalid_skill_frontmatter_schema"));
    assert!(stdout.contains("kind: repo_authoring_policy"));
    assert!(stdout.contains(
        "problem: Skill-shaped document frontmatter must satisfy the parser-owned SKILL.md schema."
    ));
    assert!(stdout.contains(
        "detail: skill frontmatter `metadata.routing_keywords` must be a non-empty string array"
    ));
    assert!(stdout.contains(
        "expected: Start from common frontmatter, then add top-level `type: skill`, `name`, `metadata.version`, `metadata.source`, and a non-empty `metadata.routing_keywords` array."
    ));
    Ok(())
}

#[test]
fn lint_accepts_skill_md_with_strict_skill_frontmatter() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("skills/demo"))?;
    std::fs::write(
        temp.path().join("skills/demo/SKILL.md"),
        strict_skill_doc(
            "Demo Skill",
            "demo-skill",
            "https://example.test/skills/demo",
            "demo",
        ),
    )?;

    let (status, stdout) = run_lint(&temp, Some("skills"))?;

    assert_eq!(status, Some(0), "{stdout}");
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
        strict_skill_doc(
            "Planner",
            "planner",
            "https://example.test/skills/planner",
            "planner",
        ),
    )?;

    let (status, stdout) = run_lint(&temp, None)?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Markdown lint passed: checked 1 file(s), 0 issue(s)."),
        "{stdout}"
    );
    Ok(())
}
