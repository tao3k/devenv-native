use anyhow::Result;
use tempfile::TempDir;

use super::{
    read_snapshot_revision, run_semantic_plan_read_model_materialization_with_args,
    run_semantic_preflight_read_model_materialization_with_args, run_semantic_snapshot_read_model,
    write_semantic_fixture,
};

#[test]
fn semantic_plan_read_model_materialization_renders_ready_plan() -> Result<()> {
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

    let args = ["--expect-snapshot", expected_revision.as_str()];
    let (status, stdout, stderr) =
        run_semantic_plan_read_model_materialization_with_args(&temp, None, &args)?;

    assert_eq!(status, Some(0), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(
        stdout.contains("Semantic read-model materialization plan ready: duckdb snapshot_swap"),
        "materialization plan should be ready: {stdout}"
    );
    assert!(
        stdout.contains("- expected: "),
        "expected snapshot should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("(matched)"),
        "expected snapshot should be marked matched: {stdout}"
    );
    assert!(
        stdout.contains("- writeback: read_model_only_no_semantic_writeback"),
        "writeback boundary should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("semantic_objects: 1 row(s), 18 column(s), materialized via duckdb_materialized_arrow_staging"),
        "table plan should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("check_expected_snapshot_revision"),
        "snapshot gate step should be included: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_plan_read_model_materialization_blocks_snapshot_mismatch() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout, stderr) = run_semantic_plan_read_model_materialization_with_args(
        &temp,
        None,
        &[
            "--expect-snapshot",
            "blake3:0000000000000000000000000000000000000000000000000000000000000000",
        ],
    )?;

    assert_eq!(status, Some(1), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(
        stdout.contains("Semantic read-model materialization plan blocked"),
        "materialization plan should be blocked: {stdout}"
    );
    assert!(
        stdout.contains("(mismatch)"),
        "snapshot mismatch should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("- snapshot: blake3:"),
        "current snapshot should be rendered: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_preflight_read_model_materialization_runs_ready_smoke_query() -> Result<()> {
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

    let args = ["--expect-snapshot", expected_revision.as_str()];
    let (status, stdout, stderr) =
        run_semantic_preflight_read_model_materialization_with_args(&temp, None, &args)?;

    assert_eq!(status, Some(0), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(
        stdout.contains(
            "Semantic read-model materialization preflight ready: target duckdb, execution datafusion"
        ),
        "preflight should be ready: {stdout}"
    );
    assert!(
        stdout.contains("(matched)"),
        "expected snapshot should be marked matched: {stdout}"
    );
    assert!(
        stdout.contains("- registered: 3 table(s), 2 row(s), 3 batch(es)"),
        "registration stats should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("- smoke result: 3 row(s) across"),
        "smoke query result should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("semantic_objects: 1 row(s), 18 column(s), materialized via datafusion_request_scoped_arrow"),
        "table preflight should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("semantic_projection_state"),
        "smoke query should mention projection-state table: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_preflight_read_model_materialization_blocks_snapshot_mismatch() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout, stderr) = run_semantic_preflight_read_model_materialization_with_args(
        &temp,
        None,
        &[
            "--expect-snapshot",
            "blake3:0000000000000000000000000000000000000000000000000000000000000000",
        ],
    )?;

    assert_eq!(status, Some(1), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(
        stdout.contains("Semantic read-model materialization preflight blocked"),
        "preflight should be blocked: {stdout}"
    );
    assert!(
        stdout.contains("(mismatch)"),
        "snapshot mismatch should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("- execution: skipped_snapshot_gate_blocked"),
        "blocked preflight should skip registration: {stdout}"
    );
    Ok(())
}
