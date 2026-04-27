use anyhow::Result;
use tempfile::TempDir;

use super::{assert_lint_text_snapshot, run_markdown_lint};

#[test]
fn markdown_lint_default_text_output_uses_compact_source_diagnostics() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::write(temp.path().join("demo.md"), "# Heading\nbody\n")?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert_lint_text_snapshot("markdown_lint_compact_missing_frontmatter", stdout.as_str());
    assert!(
        stdout
            .contains("[missing_frontmatter] Error: Document-level YAML frontmatter is required.")
    );
    assert!(stdout.contains(",-[ demo.md:1:1 ]"), "{stdout}");
    assert!(
        stdout.contains("Note 1: kind: repo_authoring_policy"),
        "{stdout}"
    );
    assert!(stdout.contains("Note 2: problem: Document-level YAML frontmatter is required."));
    assert!(
        stdout.contains("Note 3: detail: document must start with a YAML frontmatter block"),
        "{stdout}"
    );
    assert!(
        stdout.contains("Help: expected: Add a leading `--- ... ---` block"),
        "{stdout}"
    );
    assert!(
        !stdout.contains("Context:"),
        "default text output should keep metadata inside the compact diagnostic:\n{stdout}"
    );
    assert!(
        !stdout.contains("  - line 1, column 1"),
        "default text output should not fall back to the old hand-rendered bullet format:\n{stdout}"
    );
    Ok(())
}

#[test]
fn markdown_lint_invalid_utf8_fallback_keeps_notes_and_help_compact() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::write(temp.path().join("demo.md"), vec![0xff, 0xfe, 0xfd])?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert_lint_text_snapshot("markdown_lint_compact_invalid_utf8", stdout.as_str());
    assert!(stdout.contains("[invalid_utf8] Error: Markdown file is not valid UTF-8."));
    assert!(stdout.contains("--> demo.md:1:1"), "{stdout}");
    assert!(stdout.contains("Note: kind: official_syntax"), "{stdout}");
    assert!(stdout.contains("Note: problem: Markdown file is not valid UTF-8."));
    assert!(stdout.contains("Help: expected: Encode the file as UTF-8 before linting it."));
    assert!(!stdout.contains("Context:"), "{stdout}");
    Ok(())
}

#[test]
fn markdown_lint_skill_schema_snapshot_names_required_repair_fields() -> Result<()> {
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
            "date: 2026-04-26T09:30-07:00\n",
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

    let (status, stdout) = run_markdown_lint(&temp, Some("skills"))?;

    assert_eq!(status, Some(1));
    assert_lint_text_snapshot(
        "markdown_lint_compact_skill_frontmatter_schema",
        stdout.as_str(),
    );
    assert!(
        stdout.contains("invalid_skill_frontmatter_schema"),
        "{stdout}"
    );
    assert!(
        stdout.contains("top-level `type` must be `skill`"),
        "{stdout}"
    );
    assert!(stdout.contains("metadata.source"), "{stdout}");
    assert!(stdout.contains("metadata.routing_keywords"), "{stdout}");
    assert!(
        !stdout.contains("metadata.intents` must"),
        "intents is optional and must not be reported missing:\n{stdout}"
    );
    Ok(())
}

#[test]
fn markdown_lint_episteme_framework_snapshot_carries_repairable_help() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("docs/frameworks/johnny_decimal"))?;
    std::fs::write(
        temp.path()
            .join("docs/frameworks/johnny_decimal/anchor_syntax.md"),
        "---\nkind: reference\ntitle: Johnny.Decimal Anchor Syntax\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nmetadata:\n  retrieval:\n    saliency_base: 5.5\n    decay_rate: 0.05\n---\n# Johnny.Decimal Anchor Syntax\n",
    )?;
    std::fs::write(
        temp.path().join("docs/Johnny.Decimal Anchor Syntax.md"),
        "---\nkind: reference\ntitle: Johnny.Decimal Anchor Syntax\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nmetadata:\n  retrieval:\n    saliency_base: 5.5\n    decay_rate: 0.05\n---\n# Johnny.Decimal Anchor Syntax\n",
    )?;
    std::fs::write(
        temp.path().join("docs/episteme_framework_matrix.md"),
        concat!(
            "---\n",
            "kind: reference\n",
            "title: Wendao Episteme Framework Matrix\n",
            "category: docs\n",
            "tags:\n",
            "  - docs\n",
            "description: Demo note\n",
            "author: xiuxian-artisan-workshop\n",
            "date: 2026-04-26T09:30-07:00\n",
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "---\n",
            "# Wendao Episteme Framework Matrix\n",
            "Coverage: Johnny.Decimal, Diataxis, ADR, Evergreen, MOC, Folgezettel, IBIS, ",
            "search reasoning, semantic consistency, conflict arbitration -> ",
            "[[Johnny.Decimal Anchor Syntax|frameworks/johnny_decimal/anchor_syntax]].\n",
        ),
    )?;

    let (status, stdout) = run_markdown_lint(&temp, Some("docs"))?;

    assert_eq!(status, Some(1));
    assert_lint_text_snapshot(
        "markdown_lint_compact_episteme_framework_repair",
        stdout.as_str(),
    );
    assert!(
        stdout.contains("non_canonical_obsidian_alias_order"),
        "{stdout}"
    );
    assert!(
        stdout.contains(
            "Johnny.Decimal, Diataxis, ADR, Evergreen, MOC, Folgezettel, IBIS, search reasoning, semantic consistency, conflict arbitration"
        ),
        "{stdout}"
    );
    assert!(
        stdout.contains("Help 1: expected: rewrite as `[[frameworks/johnny_decimal/anchor_syntax|Johnny.Decimal Anchor Syntax]]`."),
        "{stdout}"
    );
    assert!(
        stdout.contains("Note 5: found: [[Johnny.Decimal Anchor Syntax|frameworks/johnny_decimal/anchor_syntax]]"),
        "{stdout}"
    );
    assert!(!stdout.contains("missing_local_target"), "{stdout}");
    assert!(!stdout.contains("Context:"), "{stdout}");
    Ok(())
}
