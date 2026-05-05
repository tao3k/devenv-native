//! Runtime dispatch for semantic SSOT commands.

use super::{SemanticCommand, SemanticRefreshProjectionsArgs};
use crate::lint::{
    self, SemanticLintArgs, SemanticLintProjectionValidationArgs, SemanticLintValidationArgs,
    SemanticLintWritebackArgs,
};
use crate::{ClientContext, CommandOutcome};
use anyhow::Result;
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
