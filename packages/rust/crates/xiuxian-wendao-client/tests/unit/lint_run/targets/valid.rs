use anyhow::Result;
use tempfile::TempDir;

use crate::lint_run::run_markdown_lint;

#[test]
fn lint_accepts_root_anchored_in_repo_target() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("docs"))?;
    std::fs::write(
        temp.path().join("docs/index.md"),
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nmetadata:\n  retrieval:\n    saliency_base: 5.5\n    decay_rate: 0.05\ntitle: Index\n---\n# Index\n",
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
            "See [Index](/docs/index.md).\n",
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
fn lint_accepts_existing_local_targets_and_external_urls() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("docs"))?;
    std::fs::create_dir_all(temp.path().join("assets"))?;
    std::fs::write(
        temp.path().join("docs/index.md"),
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nmetadata:\n  retrieval:\n    saliency_base: 5.5\n    decay_rate: 0.05\ntitle: Index\n---\n# Index\n",
    )?;
    std::fs::write(temp.path().join("assets/diagram.png"), b"png")?;
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
            "See [Index](docs/index).\n",
            "And ![Architecture](assets/diagram.png).\n",
            "And [OpenAI](https://openai.com/).\n",
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
fn lint_accepts_existing_wikilink_targets_with_johnny_decimal_dotted_names() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join("docs"))?;
    std::fs::write(
        temp.path().join("docs/10.01_kernel.md"),
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nmetadata:\n  retrieval:\n    saliency_base: 5.5\n    decay_rate: 0.05\ntitle: Kernel\n---\n# Kernel\n",
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
            "- [[10.01_kernel|Kernel]]\n",
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
