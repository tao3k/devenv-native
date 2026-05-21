//! Org-native SDD command execution.

use anyhow::Result;
use xiuxian_wendao_parsers::{
    OrgizeSddGraphDiffRequest, OrgizeSddStatusRequest, count_sdd_graph_drift,
    count_sdd_status_issues, render_sdd_graph_diff, render_sdd_status, render_sdd_status_json,
};

use crate::orgize::{OrgizeSddCommand, OrgizeSddGraphDiffArgs, OrgizeSddStatusArgs};
use crate::{ClientContext, CommandOutcome};

use super::paths::resolve_sdd_paths;

pub(super) fn run_sdd(
    command: &OrgizeSddCommand,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    match command {
        OrgizeSddCommand::Status(args) => run_sdd_status(args, context),
        OrgizeSddCommand::GraphDiff(args) => run_sdd_graph_diff(args, context),
    }
}

fn run_sdd_graph_diff(
    args: &OrgizeSddGraphDiffArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let request = OrgizeSddGraphDiffRequest {
        paths: resolve_sdd_paths(&args.paths, context)?,
    };
    let rendered = render_sdd_graph_diff(&request)?;
    print!("{rendered}");
    if args.fail_on_drift && count_sdd_graph_drift(&request)? > 0 {
        Ok(CommandOutcome::failure(1))
    } else {
        Ok(CommandOutcome::success())
    }
}

fn run_sdd_status(args: &OrgizeSddStatusArgs, context: &ClientContext) -> Result<CommandOutcome> {
    let request = OrgizeSddStatusRequest {
        paths: resolve_sdd_paths(&args.paths, context)?,
        issues_only: args.issues_only,
    };
    let rendered = if args.json {
        render_sdd_status_json(&request)?
    } else {
        render_sdd_status(&request)?
    };
    print!("{rendered}");
    if args.fail_on_issues && count_sdd_status_issues(&request)? > 0 {
        Ok(CommandOutcome::failure(1))
    } else {
        Ok(CommandOutcome::success())
    }
}
