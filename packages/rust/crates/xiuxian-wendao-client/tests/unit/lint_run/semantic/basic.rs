use anyhow::Result;
use tempfile::TempDir;

use super::{
    run_semantic_lint, run_semantic_lint_with_args, write_semantic_fixture,
    write_semantic_fixture_with_relation,
};

#[test]
fn semantic_lint_accepts_valid_semantic_root() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout) = run_semantic_lint(&temp, None)?;

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains(
            "Semantic lint passed: checked 1 root(s), 1 object(s), 1 projection(s), 0 change intent(s), 0 issue(s)."
        ),
        "{stdout}"
    );
    Ok(())
}

#[test]
fn semantic_lint_reports_unresolved_relations() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture_with_relation(
        &temp,
        "task.fixture",
        "task",
        "Task Fixture",
        "active",
        "  - kind: depends_on\n    target: component.missing\n",
    )?;

    let (status, stdout) = run_semantic_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(
        stdout.contains("component.missing"),
        "unresolved target should be rendered: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_lint_sql_guard_reports_stale_projection() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout) = run_semantic_lint_with_args(&temp, None, &["--semantic-sql-guard"])?;

    assert_eq!(status, Some(1));
    assert!(
        stdout.contains("SQL guard semantic_sql.projection_freshness review_required"),
        "SQL guard status should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("1 failing row(s)"),
        "SQL guard failing row count should be rendered: {stdout}"
    );
    Ok(())
}
