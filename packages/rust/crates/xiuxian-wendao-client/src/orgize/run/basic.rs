//! Basic `orgize` format and lint command execution.

use anyhow::Result;
use xiuxian_wendao_parsers::{
    OrgizeFormatRequest, OrgizeLintOutputFormat, OrgizeLintRequest, format_org_files,
    lint_org_files,
};

use crate::orgize::{OrgizeFormatArgs, OrgizeLintArgs, OrgizeLintFormatArg};
use crate::{ClientContext, CommandOutcome};

use super::paths::{display_path, resolve_paths};

pub(super) fn run_format(
    args: &OrgizeFormatArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let report = format_org_files(&OrgizeFormatRequest {
        paths: resolve_paths(&args.paths, context),
        check: args.check,
    })?;
    if args.check {
        for path in &report.changed_paths {
            eprintln!("{}: needs formatting", display_path(path, context));
        }
    }
    Ok(if args.check && report.changed() {
        CommandOutcome::failure(1)
    } else {
        CommandOutcome::success()
    })
}

pub(super) fn run_lint(args: &OrgizeLintArgs, context: &ClientContext) -> Result<CommandOutcome> {
    let output_format = if args.json {
        OrgizeLintOutputFormat::Json
    } else {
        lint_output_format(args.format)
    };
    let report = lint_org_files(&OrgizeLintRequest {
        paths: resolve_paths(&args.paths, context),
        output_format,
        priority_highest: args.priority_highest.clone(),
        priority_lowest: args.priority_lowest.clone(),
        priority_default: args.priority_default.clone(),
        fix: args.fix,
    })?;
    print!("{}", report.render(output_format));
    Ok(if report.is_clean() {
        CommandOutcome::success()
    } else {
        CommandOutcome::failure(1)
    })
}

fn lint_output_format(format: OrgizeLintFormatArg) -> OrgizeLintOutputFormat {
    match format {
        OrgizeLintFormatArg::Compact => OrgizeLintOutputFormat::Compact,
        OrgizeLintFormatArg::Text => OrgizeLintOutputFormat::Text,
        OrgizeLintFormatArg::Json => OrgizeLintOutputFormat::Json,
    }
}
