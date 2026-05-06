use anyhow::Result;
use tempfile::TempDir;

use super::{
    run_semantic_lint, run_semantic_lint_with_args, write_pending_semantic_lifecycle_fixture,
    write_semantic_lifecycle_fixture,
};

#[test]
fn semantic_lint_reports_lifecycle_plan() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_lifecycle_fixture(&temp)?;

    let (status, stdout) = run_semantic_lint_with_args(&temp, None, &["--lifecycle-plan"])?;

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains(
            "Lifecycle plan 1 promotion(s), 0 demotion(s), 0 other transition(s), 0 pending apply target(s), 1 already-applied writeback target(s), 0 blocked target(s)."
        ),
        "lifecycle plan summary should be rendered: {stdout}"
    );
    assert!(
        stdout.contains(
            "change.fixture.lifecycle: task.accepted candidate -> active (promotion, already_applied)"
        ),
        "lifecycle plan entry should be rendered: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_lint_applies_pending_lifecycle_plan() -> Result<()> {
    let temp = TempDir::new()?;
    write_pending_semantic_lifecycle_fixture(&temp)?;

    let (status, stdout) = run_semantic_lint_with_args(&temp, None, &["--apply-lifecycle-plan"])?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Applied 1 semantic lifecycle writeback(s)."),
        "apply count should be rendered: {stdout}"
    );
    assert!(
        stdout.contains(
            "Lifecycle plan 1 promotion(s), 0 demotion(s), 0 other transition(s), 0 pending apply target(s), 1 already-applied writeback target(s), 0 blocked target(s)."
        ),
        "post-apply lifecycle plan should be rendered: {stdout}"
    );

    let object = std::fs::read_to_string(temp.path().join("semantic/objects/task/accepted.md"))?;
    assert!(
        object.contains("status: active"),
        "object status should be promoted: {object}"
    );
    assert!(
        object.contains("source: human_signed"),
        "promotion should update confidence source: {object}"
    );
    assert!(
        !object.contains("source: llm_suggested"),
        "promoted object must not keep llm_suggested confidence: {object}"
    );
    let intent = std::fs::read_to_string(temp.path().join("semantic/change-intents/lifecycle.md"))?;
    assert!(
        intent.contains("candidate_suggestions: []"),
        "promoted object should be removed from candidate suggestions: {intent}"
    );

    let (status, stdout) = run_semantic_lint(&temp, None)?;
    assert_eq!(status, Some(0), "{stdout}");
    Ok(())
}
