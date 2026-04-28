use anyhow::Result;
use tempfile::TempDir;
use xiuxian_wendao_client::MarkdownLintReport;

use super::run_markdown_lint_with_output;

#[test]
fn markdown_lint_emits_json_output_when_requested() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::write(
        temp.path().join("guide.md"),
        "---\nkind: reference\ncategory: docs\ntags:\n  - docs\ndescription: Demo note\nauthor: xiuxian-artisan-workshop\ndate: 2026-04-26T09:30-07:00\nmetadata:\n  retrieval:\n    saliency_base: 5.5\n    decay_rate: 0.05\ntitle: Guide\n---\n# Guide\n\nAll clear.\n",
    )?;

    let (status, stdout) = run_markdown_lint_with_output(&temp, None, Some("json"))?;
    let report: MarkdownLintReport = serde_json::from_str(stdout.as_str())?;

    assert_eq!(status, Some(0));
    assert_eq!(report.checked_files, 1);
    assert_eq!(report.files_with_issues, 0);
    assert_eq!(report.issue_count, 0);
    assert!(report.files.is_empty());
    Ok(())
}
