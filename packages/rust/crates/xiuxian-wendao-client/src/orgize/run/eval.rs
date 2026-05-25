//! Org Babel eval-contract command execution.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use xiuxian_wendao_parsers::{
    OrgizeEvalPatchRequest, OrgizeEvalPlanRequest, render_eval_patch, render_eval_plan,
};

use crate::orgize::command::{OrgizeEvalCommand, OrgizeEvalPatchArgs, OrgizeEvalPlanArgs};
use crate::{ClientContext, CommandOutcome};

pub(super) fn run_eval(
    command: &OrgizeEvalCommand,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    match command {
        OrgizeEvalCommand::Plan(args) => run_eval_plan(args, context),
        OrgizeEvalCommand::Patch(args) => run_eval_patch(args, context),
    }
}

fn run_eval_plan(args: &OrgizeEvalPlanArgs, context: &ClientContext) -> Result<CommandOutcome> {
    print!(
        "{}",
        render_eval_plan(&OrgizeEvalPlanRequest {
            name: args.name.clone(),
            path: resolve_path(&args.path, context),
            json: args.json,
        })?
    );
    Ok(CommandOutcome::success())
}

fn run_eval_patch(args: &OrgizeEvalPatchArgs, context: &ClientContext) -> Result<CommandOutcome> {
    let path = resolve_path(&args.path, context);
    print!(
        "{}",
        render_eval_patch(&OrgizeEvalPatchRequest {
            name: args.name.clone(),
            path,
            stdout: resolve_optional_text(
                args.stdout.as_deref(),
                args.stdout_file.as_ref(),
                context,
            )?,
            stderr: resolve_optional_text(
                args.stderr.as_deref(),
                args.stderr_file.as_ref(),
                context,
            )?,
            exit_code: args.exit_code,
            write: args.write,
            json: args.json,
        })?
    );
    Ok(CommandOutcome::success())
}

fn resolve_path(path: &Path, context: &ClientContext) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        context.root().join(path)
    }
}

fn resolve_optional_text(
    inline: Option<&str>,
    file: Option<&PathBuf>,
    context: &ClientContext,
) -> Result<String> {
    if let Some(value) = inline {
        return Ok(value.to_string());
    }
    let Some(path) = file else {
        return Ok(String::new());
    };
    let path = resolve_path(path, context);
    fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))
}
