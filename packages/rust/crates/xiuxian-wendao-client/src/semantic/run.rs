//! Runtime dispatch for semantic SSOT commands.

use super::{SemanticCommand, SemanticRefreshProjectionsArgs};
use crate::lint::{
    self, SemanticLintArgs, SemanticLintProjectionValidationArgs, SemanticLintValidationArgs,
    SemanticLintWritebackArgs,
};
use crate::{ClientContext, CommandOutcome};
use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub(crate) fn run_command(
    command: &SemanticCommand,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    match command {
        SemanticCommand::RefreshProjections(args) => run_refresh_projections_worker(args, context),
    }
}

fn run_refresh_projections_worker(
    args: &SemanticRefreshProjectionsArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    if args.require_clean_worktree {
        ensure_clean_git_worktree(context.root())?;
    }

    let mut completed_runs = 0_usize;
    loop {
        let outcome = run_refresh_projections_worker_pass(args, context)?;
        completed_runs += 1;
        if outcome.exit_code() != 0 {
            return Ok(outcome);
        }
        if args
            .max_runs
            .is_some_and(|max_runs| completed_runs >= max_runs.get())
        {
            return Ok(outcome);
        }
        if args.interval_secs == 0 && args.max_runs.is_none() {
            return Ok(outcome);
        }
        if args.interval_secs > 0 {
            thread::sleep(Duration::from_secs(args.interval_secs));
        }
    }
}

fn run_refresh_projections_worker_pass(
    args: &SemanticRefreshProjectionsArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let lint_args = SemanticLintArgs {
        validation: SemanticLintValidationArgs {
            read_model_summary: false,
            semantic_sql_guard: false,
            lifecycle_plan: false,
            projection: SemanticLintProjectionValidationArgs {
                projection_refresh_plan: true,
                require_fresh_projections: true,
            },
        },
        writeback: SemanticLintWritebackArgs {
            refresh_projections: true,
            apply_lifecycle_plan: false,
        },
        paths: args.paths.clone(),
    };
    lint::run_semantic_lint(&lint_args, context)
}

fn ensure_clean_git_worktree(root: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["status", "--porcelain"])
        .output()
        .with_context(|| {
            format!(
                "failed to run git clean-worktree check at `{}`",
                root.display()
            )
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "semantic refresh supervisor clean-worktree check requires a git worktree at `{}`: {}",
            root.display(),
            stderr.trim()
        );
    }
    if !output.stdout.is_empty() {
        let status = String::from_utf8_lossy(&output.stdout);
        bail!(
            "semantic refresh supervisor clean-worktree check requires a clean git worktree at `{}`; pending changes:\n{}",
            root.display(),
            status.trim_end()
        );
    }
    Ok(())
}
