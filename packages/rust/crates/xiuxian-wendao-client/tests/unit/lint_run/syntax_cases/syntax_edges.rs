use anyhow::Result;
use tempfile::TempDir;

use super::{common_doc, run_lint};

#[test]
fn lint_reports_unclosed_frontmatter() -> Result<()> {
    let temp = TempDir::new()?;
    std::fs::write(
        temp.path().join("demo.md"),
        concat!(
            "---\n",
            "kind: reference\n",
            "category: docs\n",
            "tags:\n",
            "  - docs\n",
            "description: Demo note\n",
            "author: xiuxian-artisan-workshop\n",
            "date: \"2026-04-26T09:30-07:00\"\n",
            "saliency_base: 5.5\n",
            "decay_rate: 0.05\n",
            "title: demo\n",
            "body\n",
        ),
    )?;

    let (status, stdout) = run_lint(&temp, None)?;

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

    let (status, stdout) = run_lint(&temp, None)?;

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
        format!("{}{}\n", common_doc("Demo"), "```rust\nfn main() {}"),
    )?;

    let (status, stdout) = run_lint(&temp, None)?;

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
        format!("{}{}\n", common_doc("Demo"), "```rust\nfn main() {}\n```"),
    )?;

    let (status, stdout) = run_lint(&temp, None)?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Markdown lint passed: checked 1 file(s), 0 issue(s)."),
        "{stdout}"
    );
    Ok(())
}
