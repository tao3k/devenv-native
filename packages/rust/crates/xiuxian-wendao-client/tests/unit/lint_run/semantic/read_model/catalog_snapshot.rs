use anyhow::Result;
use tempfile::TempDir;

use super::{
    read_snapshot_revision, run_semantic_check_read_model_snapshot_with_args,
    run_semantic_describe_read_model, run_semantic_lint_with_args,
    run_semantic_snapshot_read_model, write_semantic_fixture,
};

#[test]
fn semantic_lint_renders_read_model_summary() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout) = run_semantic_lint_with_args(&temp, None, &["--read-model-summary"])?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Read-model summary projected"),
        "read-model status should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("semantic_objects 1 row(s)"),
        "object row count should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("semantic_projection_state 1 row(s)"),
        "projection-state row count should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("repo-native semantic artifacts remain authoritative"),
        "authority boundary should be rendered: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_describe_read_model_renders_catalog() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout) = run_semantic_describe_read_model(&temp, None)?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Semantic read-model catalog: 3 table(s), 2 row(s)"),
        "catalog summary should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("semantic_objects: 1 row(s), 18 column(s)"),
        "semantic object table should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("  - id: Utf8 not null"),
        "column metadata should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("semantic_projection_state: 1 row(s), 9 column(s)"),
        "projection-state table should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("repo_native_semantic_artifacts"),
        "authority boundary should be rendered: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_snapshot_read_model_renders_revisions() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout) = run_semantic_snapshot_read_model(&temp, None)?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Semantic read-model snapshot: blake3:"),
        "snapshot revision should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("tables: 3 table(s), 2 row(s)"),
        "snapshot summary should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("semantic_objects: 1 row(s), 18 column(s), revision blake3:"),
        "object table revision should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("semantic_projection_state: 1 row(s), 9 column(s), revision blake3:"),
        "projection-state table revision should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("repo_native_semantic_artifacts"),
        "authority boundary should be rendered: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_check_read_model_snapshot_accepts_expected_revision() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;
    let (status, snapshot_stdout) = run_semantic_snapshot_read_model(&temp, None)?;
    assert_eq!(status, Some(0), "{snapshot_stdout}");
    let expected_revision = read_snapshot_revision(&snapshot_stdout)?;

    let args = ["--expect", expected_revision.as_str()];
    let (status, stdout, stderr) =
        run_semantic_check_read_model_snapshot_with_args(&temp, None, &args)?;

    assert_eq!(status, Some(0), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(
        stdout.contains("Semantic read-model snapshot check passed"),
        "snapshot check should pass: {stdout}"
    );
    assert!(
        stdout.contains(expected_revision.as_str()),
        "expected revision should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("semantic_objects: 1 row(s), revision blake3:"),
        "table revisions should be rendered: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_check_read_model_snapshot_rejects_mismatch() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout, stderr) = run_semantic_check_read_model_snapshot_with_args(
        &temp,
        None,
        &[
            "--expect",
            "blake3:0000000000000000000000000000000000000000000000000000000000000000",
        ],
    )?;

    assert_eq!(status, Some(1), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(
        stdout.contains("Semantic read-model snapshot check failed"),
        "snapshot check should fail: {stdout}"
    );
    assert!(
        stdout.contains(
            "- expected: blake3:0000000000000000000000000000000000000000000000000000000000000000"
        ),
        "expected revision should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("- current: blake3:"),
        "current revision should be rendered: {stdout}"
    );
    Ok(())
}
