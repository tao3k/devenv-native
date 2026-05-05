//! Runtime dispatch for semantic SSOT commands.

use super::{SemanticCommand, SemanticRefreshProjectionsArgs};
use crate::lint::{
    self, SemanticLintArgs, SemanticLintProjectionValidationArgs, SemanticLintValidationArgs,
    SemanticLintWritebackArgs,
};
use crate::{ClientContext, CommandOutcome};
use anyhow::Result;

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
