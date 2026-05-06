use anyhow::Result;
use tempfile::TempDir;

use crate::lint_run::run_markdown_lint;

#[test]
fn lint_reports_local_target_outside_root() -> Result<()> {
    let temp = TempDir::new()?;
    let workspace = temp.path().join("workspace");
    std::fs::create_dir_all(workspace.join("docs"))?;
    std::fs::write(
        temp.path().join("outside.md"),
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nsaliency_base: 5.5\ndecay_rate: 0.05\ntitle: Outside\n---\n",
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
            "saliency_base: 5.5\n",
            "decay_rate: 0.05\n",
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
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nsaliency_base: 5.5\ndecay_rate: 0.05\ntitle: Internal Artifact\n---\n# Internal Artifact\n## Stable Heading\n",
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
