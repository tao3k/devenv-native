use anyhow::Result;
use tempfile::TempDir;

use super::{
    run_semantic_query_read_model_with_args, run_semantic_query_read_model_with_args_and_stderr,
    write_semantic_fixture,
};

#[test]
fn semantic_query_read_model_runs_sql() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout) = run_semantic_query_read_model_with_args(
        &temp,
        None,
        &[
            "--query",
            "select id, kind from semantic_objects order by id",
        ],
    )?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Semantic read-model query returned 1 row(s)"),
        "query row count should be rendered: {stdout}"
    );
    assert!(
        stdout.contains(
            "registered tables: semantic_objects, semantic_relations, semantic_projection_state"
        ),
        "registered read-model tables should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("id=decision.fixture, kind=decision"),
        "query rows should be rendered: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_query_read_model_rejects_mutation_sql() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout, stderr) = run_semantic_query_read_model_with_args_and_stderr(
        &temp,
        None,
        &[
            "--query",
            "insert into semantic_objects (id) values ('decision.bad')",
        ],
    )?;

    assert_eq!(status, Some(1), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(
        stderr.contains("read-only query statement"),
        "mutation rejection should explain the read-only contract: {stderr}"
    );
    Ok(())
}
