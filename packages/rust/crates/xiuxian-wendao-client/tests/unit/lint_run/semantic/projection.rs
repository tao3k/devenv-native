use anyhow::Result;
use tempfile::TempDir;

use super::{
    run_semantic_lint, run_semantic_lint_with_args, write_semantic_fixture,
    write_semantic_lifecycle_fixture,
};

#[test]
fn semantic_lint_refreshes_projection_source_revision() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout) = run_semantic_lint_with_args(&temp, None, &["--refresh-projections"])?;

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains("Refreshed 1 semantic projection source revision(s)."),
        "refresh count should be rendered: {stdout}"
    );
    let projection =
        std::fs::read_to_string(temp.path().join("semantic/projections/llm-compression.md"))?;
    assert!(
        !projection.contains("source_revision: stale-fixture"),
        "stale source revision should be replaced: {projection}"
    );
    assert!(
        projection.contains("staleness: fresh"),
        "staleness should be marked fresh: {projection}"
    );
    assert!(
        projection.contains("source_objects:\n  - decision.fixture"),
        "projection refresh should preserve block sequence indentation: {projection}"
    );
    assert!(
        projection.contains("source_revision: \"blake3:"),
        "projection refresh should keep source revision quoted: {projection}"
    );
    assert!(
        projection.contains("projection_revision: test.v1"),
        "projection revision should remain unchanged: {projection}"
    );

    let (status, stdout) = run_semantic_lint(&temp, None)?;
    assert_eq!(status, Some(0), "{stdout}");
    Ok(())
}

#[test]
fn semantic_lint_requires_fresh_projection_refresh_targets() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_lifecycle_fixture(&temp)?;

    let (status, stdout) =
        run_semantic_lint_with_args(&temp, None, &["--require-fresh-projections"])?;

    assert_eq!(status, Some(1));
    assert!(
        stdout.contains("1 projection policy issue(s)"),
        "projection policy issue count should be rendered: {stdout}"
    );
    assert!(
        stdout.contains(
            "Projection freshness policy semantic_projection.required_refresh_targets review_required"
        ),
        "projection policy failure should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("llm_compression (stale, stale)"),
        "stale projection entry should be rendered: {stdout}"
    );

    let (status, stdout) = run_semantic_lint_with_args(
        &temp,
        None,
        &["--refresh-projections", "--require-fresh-projections"],
    )?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Refreshed 1 semantic projection source revision(s)."),
        "refresh count should be rendered: {stdout}"
    );
    assert!(
        stdout.contains(
            "Projection freshness policy semantic_projection.required_refresh_targets passed (0 failing projection(s))"
        ),
        "projection policy pass should be rendered: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_lint_renders_projection_refresh_plan() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout) =
        run_semantic_lint_with_args(&temp, None, &["--projection-refresh-plan"])?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Projection refresh plan refresh_required"),
        "projection refresh plan should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("llm_compression -> refresh_source_revision (stale, stale)"),
        "projection refresh entry should be rendered: {stdout}"
    );

    let (status, stdout) = run_semantic_lint_with_args(
        &temp,
        None,
        &["--refresh-projections", "--projection-refresh-plan"],
    )?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Projection refresh plan up_to_date (0 refreshable projection(s))"),
        "refreshed projection should make plan empty: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_lint_renders_projection_refresh_plan_for_fresh_revision_mismatch() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;
    let projection_path = temp.path().join("semantic/projections/llm-compression.md");
    let projection = std::fs::read_to_string(&projection_path)?;
    std::fs::write(
        &projection_path,
        projection.replace("staleness: stale", "staleness: fresh"),
    )?;

    let (status, stdout) =
        run_semantic_lint_with_args(&temp, None, &["--projection-refresh-plan"])?;

    assert_eq!(status, Some(1), "{stdout}");
    assert!(
        stdout.contains("Projection refresh plan refresh_required"),
        "projection refresh plan should render even for refreshable validation issues: {stdout}"
    );
    assert!(
        stdout.contains("llm_compression -> refresh_source_revision"),
        "projection refresh entry should be rendered: {stdout}"
    );
    Ok(())
}
