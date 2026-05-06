use anyhow::Result;
use tempfile::TempDir;

use super::{
    initialize_git_fixture, run_semantic_refresh_projections,
    run_semantic_refresh_projections_with_args,
    run_semantic_refresh_projections_with_args_and_stderr, write_semantic_fixture,
};

#[test]
fn semantic_refresh_projections_command_runs_one_worker_pass() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout) = run_semantic_refresh_projections(&temp, None)?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Refreshed 1 semantic projection source revision(s)."),
        "worker should refresh stale projection metadata: {stdout}"
    );
    assert!(
        stdout.contains("Projection refresh plan up_to_date"),
        "worker should report an empty post-refresh plan: {stdout}"
    );
    assert!(
        stdout.contains(
            "Projection freshness policy semantic_projection.required_refresh_targets passed"
        ),
        "worker should enforce post-refresh projection freshness: {stdout}"
    );

    let projection =
        std::fs::read_to_string(temp.path().join("semantic/projections/llm-compression.md"))?;
    assert!(
        projection.contains("staleness: fresh"),
        "worker should mark projection metadata fresh: {projection}"
    );
    Ok(())
}

#[test]
fn semantic_refresh_projections_command_runs_bounded_repeated_worker_passes() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout) = run_semantic_refresh_projections_with_args(
        &temp,
        None,
        &["--interval-secs", "0", "--max-runs", "2"],
    )?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Refreshed 1 semantic projection source revision(s)."),
        "first worker pass should refresh stale projection metadata: {stdout}"
    );
    assert_eq!(
        stdout.matches("Projection refresh plan up_to_date").count(),
        2,
        "bounded runner should render a post-refresh plan for each pass: {stdout}"
    );
    assert_eq!(
        stdout
            .matches(
                "Projection freshness policy semantic_projection.required_refresh_targets passed"
            )
            .count(),
        2,
        "bounded runner should enforce freshness for each pass: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_refresh_projections_clean_worktree_guard_accepts_clean_git_root() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;
    initialize_git_fixture(&temp)?;

    let (status, stdout, stderr) = run_semantic_refresh_projections_with_args_and_stderr(
        &temp,
        None,
        &["--require-clean-worktree"],
    )?;

    assert_eq!(status, Some(0), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(
        stdout.contains("Refreshed 1 semantic projection source revision(s)."),
        "clean root should allow supervised refresh: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_refresh_projections_clean_worktree_guard_rejects_dirty_git_root() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;
    initialize_git_fixture(&temp)?;
    std::fs::write(temp.path().join("dirty.md"), "# Dirty\n")?;

    let (status, stdout, stderr) = run_semantic_refresh_projections_with_args_and_stderr(
        &temp,
        None,
        &["--require-clean-worktree"],
    )?;

    assert_eq!(status, Some(1), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(
        stderr.contains("requires a clean git worktree"),
        "dirty root should be rejected before refresh: {stderr}"
    );
    assert!(
        stderr.contains("dirty.md"),
        "dirty path should be rendered for supervisor triage: {stderr}"
    );
    Ok(())
}
