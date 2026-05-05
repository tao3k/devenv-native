use anyhow::Result;
use tempfile::TempDir;

use crate::lint_run::run_markdown_lint;

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
            "saliency_base: 5.5\n",
            "decay_rate: 0.05\n",
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
            "saliency_base: 5.5\n",
            "decay_rate: 0.05\n",
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
            "saliency_base: 5.5\n",
            "decay_rate: 0.05\n",
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
