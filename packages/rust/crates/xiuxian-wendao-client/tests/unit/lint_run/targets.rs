use anyhow::Result;
use tempfile::TempDir;

use super::run_markdown_lint;

#[test]
fn lint_reports_missing_markdown_link_target() -> Result<()> {
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
            "See [Missing](docs/missing-note).\n",
        ),
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("missing_local_target"));
    assert!(stdout.contains("target: docs/missing-note"));
    assert!(stdout.contains("found: [Missing](docs/missing-note)"));
    Ok(())
}

#[test]
fn lint_reports_missing_wikilink_target() -> Result<()> {
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
            "See [[docs/missing-note|Missing]].\n",
        ),
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("missing_local_target"));
    assert!(stdout.contains("target: docs/missing-note"));
    assert!(stdout.contains("found: [[docs/missing-note|Missing]]"));
    Ok(())
}

#[test]
fn lint_reports_missing_attachment_target() -> Result<()> {
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
            "See ![Architecture](assets/diagram.png).\n",
        ),
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("missing_local_target"));
    assert!(stdout.contains("target: assets/diagram.png"));
    assert!(stdout.contains("found: ![Architecture](assets/diagram.png)"));
    Ok(())
}

#[test]
fn lint_reports_local_target_outside_root() -> Result<()> {
    let temp = TempDir::new()?;
    let workspace = temp.path().join("workspace");
    std::fs::create_dir_all(workspace.join("docs"))?;
    std::fs::write(
        temp.path().join("outside.md"),
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nmetadata:\n  retrieval:\n    saliency_base: 5.5\n    decay_rate: 0.05\ntitle: Outside\n---\n",
    )?;
    std::fs::write(
        workspace.join("docs/guide.md"),
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
            "See [Outside](../../outside.md).\n",
        ),
    )?;

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(&workspace)
        .arg("lint")
        .arg("markdown")
        .output()?;
    let status = output.status.code();
    let stdout = String::from_utf8(output.stdout)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("local_target_outside_root"));
    assert!(stdout.contains("target: ../../outside.md"));
    assert!(stdout.contains("found: [Outside](../../outside.md)"));
    Ok(())
}

#[test]
fn lint_reports_local_target_inside_transient_repo_dir() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::create_dir_all(temp.path().join(".data"))?;
    std::fs::write(
        temp.path().join(".data/internal.md"),
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nmetadata:\n  retrieval:\n    saliency_base: 5.5\n    decay_rate: 0.05\ntitle: Internal Artifact\n---\n# Internal Artifact\n## Stable Heading\n",
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
            "See [.data](.data/internal.md#Stable Heading).\n",
        ),
    )?;

    let (status, stdout) = run_markdown_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(stdout.contains("local_target_transient_dir"));
    assert!(stdout.contains("target: .data/internal.md#Stable Heading"));
    assert!(stdout.contains("found: [.data](.data/internal.md#Stable Heading)"));
    assert!(stdout.contains("transient/generated directory `.data`"));
    assert!(!stdout.contains("missing_local_fragment"));
    Ok(())
}

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
