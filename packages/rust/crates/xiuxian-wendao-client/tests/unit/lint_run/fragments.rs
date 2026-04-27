use anyhow::Result;
use tempfile::TempDir;

use super::run_markdown_lint;

#[test]
fn lint_reports_missing_cross_note_heading_fragment() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("docs"))?;
    std::fs::write(
        temp.path().join("docs/index.md"),
        concat!(
            "---\n",
            "kind: reference\n",
            "title: Documentation Index\n",
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
            "# Documentation Index\n",
            "## Existing Section\n",
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
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "---\n",
            "# Guide\n",
            "See [[docs/index#Missing Section|Missing Section]].\n",
        ),
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("missing_local_fragment"));
    assert!(stdout.contains("target: docs/index#Missing Section"));
    assert!(stdout.contains("target_title: Documentation Index"));
    assert!(stdout.contains("target_heading: Missing Section"));
    Ok(())
}

#[test]
fn lint_reports_missing_local_markdown_heading_anchor() -> Result<()> {
    let temp = TempDir::new()?;
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
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "---\n",
            "# Guide\n",
            "## Existing Heading\n",
            "See [Missing](#missing-heading).\n",
        ),
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("missing_local_fragment"));
    assert!(stdout.contains("target: #missing-heading"));
    assert!(stdout.contains("found: [Missing](#missing-heading)"));
    Ok(())
}

#[test]
fn lint_reports_missing_block_fragment() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("docs"))?;
    std::fs::write(
        temp.path().join("docs/index.md"),
        concat!(
            "---\n",
            "kind: reference\n",
            "title: Documentation Index\n",
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
            "# Documentation Index\n",
            "Paragraph.\n",
            "^real-block\n",
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
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "---\n",
            "# Guide\n",
            "See [[docs/index#^missing-block|Missing Block]].\n",
        ),
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("missing_local_fragment"));
    assert!(stdout.contains("target: docs/index#^missing-block"));
    assert!(stdout.contains("target_heading: ^missing-block"));
    assert!(stdout.contains("block anchor"));
    Ok(())
}

#[test]
fn lint_accepts_valid_markdown_and_obsidian_fragments() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("docs"))?;
    std::fs::create_dir_all(temp.path().join("notes"))?;
    std::fs::write(
        temp.path().join("docs/index.md"),
        concat!(
            "---\n",
            "kind: reference\n",
            "title: Documentation Index\n",
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
            "# Documentation Index\n",
            "## Parser Contracts\n",
            "Paragraph.\n",
            "^block-id\n",
        ),
    )?;
    std::fs::write(
        temp.path().join("guide.md"),
        concat!(
            "---\n",
            "kind: reference\n",
            "title: Markdown Guide\n",
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
            "# Markdown Guide\n",
            "## Local Heading\n",
            "Paragraph.\n",
            "^local-block-id\n",
            "See [Contracts](docs/index#parser-contracts).\n",
            "And [Local Heading](#local-heading).\n",
            "And [Block Context](docs/index#^block-id).\n",
            "And [Local Block](#^local-block-id).\n",
        ),
    )?;
    std::fs::write(
        temp.path().join("notes/guide.md"),
        concat!(
            "---\n",
            "kind: reference\n",
            "title: Obsidian Guide\n",
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
            "# Obsidian Guide\n",
            "## Local Heading\n",
            "Paragraph.\n",
            "^local-block-id\n",
            "And [[docs/index#Parser Contracts|Contracts Note]].\n",
            "And [[docs/index#^block-id|Block Context]].\n",
            "And [[#^local-block-id|Local Block]].\n",
        ),
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains("Markdown lint passed: checked 3 file(s), 0 issue(s)."),
        "{stdout}"
    );
    Ok(())
}

#[test]
fn lint_accepts_markdown_heading_fragments_with_spaces() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("docs"))?;
    std::fs::write(
        temp.path().join("docs/index.md"),
        concat!(
            "---\n",
            "kind: reference\n",
            "title: Documentation Index\n",
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
            "# Documentation Index\n",
            "## Parser Contracts\n",
        ),
    )?;
    std::fs::write(
        temp.path().join("guide.md"),
        concat!(
            "---\n",
            "kind: reference\n",
            "title: Markdown Guide\n",
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
            "# Markdown Guide\n",
            "See [Contracts](docs/index.md#Parser Contracts).\n",
        ),
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains("Markdown lint passed: checked 2 file(s), 0 issue(s)."),
        "{stdout}"
    );
    Ok(())
}

#[test]
fn lint_accepts_wikilink_fragments_for_johnny_decimal_dotted_names() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("docs"))?;
    std::fs::write(
        temp.path().join("docs/10.01_kernel.md"),
        concat!(
            "---\n",
            "kind: reference\n",
            "title: Kernel\n",
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
            "# Kernel\n",
            "## Stable Heading\n",
        ),
    )?;
    std::fs::write(
        temp.path().join("docs/10.00_moc.md"),
        concat!(
            "---\n",
            "kind: reference\n",
            "title: Map of Content\n",
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
            "# Map of Content\n",
            "- [[10.01_kernel#Stable Heading|Kernel Heading]]\n",
        ),
    )?;

    let (status, stdout) = run_markdown_lint(&temp, Some("docs"))?;

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains("Markdown lint passed: checked 2 file(s), 0 issue(s)."),
        "{stdout}"
    );
    Ok(())
}
