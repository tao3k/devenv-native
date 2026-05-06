use anyhow::Result;
use tempfile::TempDir;

use super::run_markdown_lint;

#[test]
fn lint_reports_non_canonical_obsidian_wikilinks_as_text() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("01_core"))?;
    std::fs::write(
        temp.path().join("01_core/106_docs_maintenance_playbook.md"),
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nsaliency_base: 5.5\ndecay_rate: 0.05\ntitle: Docs Maintenance Playbook\n---\n# Docs Maintenance Playbook\n",
    )?;
    std::fs::write(
        temp.path().join("guide.md"),
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nsaliency_base: 5.5\ndecay_rate: 0.05\ntitle: Guide\n---\n# Heading\nSee [[Docs Maintenance Playbook|01_core/106_docs_maintenance_playbook]].\n",
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("non_canonical_obsidian_alias_order"));
    assert!(stdout.contains("kind: repo_authoring_policy"));
    assert!(stdout.contains("target_title: Docs Maintenance Playbook"));
    assert!(stdout.contains(
        "rewrite as `[[01_core/106_docs_maintenance_playbook|Docs Maintenance Playbook]]`."
    ));
    Ok(())
}

#[test]
fn lint_reports_bare_obsidian_wikilinks_as_text() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("docs"))?;
    std::fs::write(
        temp.path().join("docs/index.md"),
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nsaliency_base: 5.5\ndecay_rate: 0.05\ntitle: Documentation Index\n---\n# Documentation Index\n",
    )?;
    std::fs::write(
        temp.path().join("guide.md"),
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nsaliency_base: 5.5\ndecay_rate: 0.05\ntitle: Guide\n---\n# Heading\nSee [[docs/index]].\n",
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("bare_obsidian_wikilink"));
    assert!(stdout.contains("kind: repo_authoring_policy"));
    assert!(stdout.contains("Obsidian officially allows bare wikilinks"));
    assert!(stdout.contains("target_title: Documentation Index"));
    assert!(stdout.contains(
        "rewrite as `[[docs/index|Documentation Index]]` or `[Documentation Index](docs/index)`."
    ));
    Ok(())
}

#[test]
fn lint_reports_redundant_heading_labels_with_namespace_guidance() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("02_parser"))?;
    std::fs::write(
        temp.path().join("02_parser/index.md"),
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nsaliency_base: 5.5\ndecay_rate: 0.05\ntitle: Wendao Parser Docs\n---\n# Wendao Parser Docs\n",
    )?;
    std::fs::write(
        temp.path().join("guide.md"),
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nsaliency_base: 5.5\ndecay_rate: 0.05\ntitle: Guide\n---\n# Heading\nSee [[02_parser/index#Semantic Check|Semantic Check]].\n",
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("redundant_obsidian_label"));
    assert!(stdout.contains("kind: repo_authoring_policy"));
    assert!(stdout.contains("target_title: Wendao Parser Docs"));
    assert!(stdout.contains("target_heading: Semantic Check"));
    assert!(stdout.contains("rewrite as `[[02_parser/index#Semantic Check|Wendao Parser Docs / Semantic Check]]` or `[Wendao Parser Docs / Semantic Check](02_parser/index#Semantic Check)`."));
    Ok(())
}

#[test]
fn lint_reports_mixed_link_syntax_as_official_syntax_failure() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("docs"))?;
    std::fs::write(
        temp.path().join("docs/index.md"),
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nsaliency_base: 5.5\ndecay_rate: 0.05\ntitle: Documentation Index\n---\n# Documentation Index\n",
    )?;
    std::fs::write(
        temp.path().join("guide.md"),
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nsaliency_base: 5.5\ndecay_rate: 0.05\ntitle: Guide\n---\n# Heading\nSee [[docs/index]](Documentation Index).\n",
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("mixed_wikilink_markdown_link"));
    assert!(stdout.contains("kind: official_syntax"));
    assert!(stdout.contains(
        "Choose either official Obsidian wikilink syntax or standard Markdown link syntax."
    ));
    Ok(())
}

#[test]
fn lint_accepts_official_obsidian_embeds_and_addressed_targets() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("docs"))?;
    std::fs::write(
        temp.path().join("docs/index.md"),
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nsaliency_base: 5.5\ndecay_rate: 0.05\ntitle: Documentation Index\n---\n# Documentation Index\n## Parser Contracts\nParagraph.\n^block-id\n",
    )?;
    std::fs::write(
        temp.path().join("Three laws of motion.md"),
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nsaliency_base: 5.5\ndecay_rate: 0.05\ntitle: Three laws of motion\n---\n# Three laws of motion\n",
    )?;
    std::fs::write(
        temp.path().join("Help and support.md"),
        concat!(
            "---\n",
            "kind: reference\n",
            "title: Help and support\n",
            "category: docs\n",
            "tags:\n",
            "  - docs\n",
            "description: Demo note\n",
            "author: xiuxian-artisan-workshop\n",
            "date: 2026-04-26T09:30-07:00\n",
            "saliency_base: 5.5\n",
            "decay_rate: 0.05\n",
            "---\n",
            "# Help and support\n",
            "## Questions and advice\n",
            "### Report bugs and request features\n",
        ),
    )?;
    std::fs::write(
        temp.path().join("guide.md"),
        concat!(
            "---\n",
            "kind: reference\n",
            "title: Guide\n",
            "category: docs\n",
            "tags:\n",
            "  - docs\n",
            "description: Demo note\n",
            "author: xiuxian-artisan-workshop\n",
            "date: 2026-04-26T09:30-07:00\n",
            "saliency_base: 5.5\n",
            "decay_rate: 0.05\n",
            "---\n",
            "# Root\n",
            "## Local Heading\n",
            "Paragraph.\n",
            "^local-block-id\n",
            "See [[Three laws of motion|Overview]].\n",
            "And [[Three laws of motion.md|Laws Note]].\n",
            "See [[docs/index#Parser Contracts|Contracts Overview]].\n",
            "And [[#Local Heading|Local Heading Context]].\n",
            "And [[#^local-block-id|Local Block Context]].\n",
            "And [[Help and support#Questions and advice#Report bugs and request features|Bug Reports]].\n",
            "And [[docs/index#^block-id|Block Context]].\n",
            "And ![[docs/index]].\n",
            "And ![[docs/index#Parser Contracts]].\n",
            "And ![[docs/index#^block-id]].\n",
        ),
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains("Markdown lint passed: checked 4 file(s), 0 issue(s)."),
        "{stdout}"
    );
    Ok(())
}
