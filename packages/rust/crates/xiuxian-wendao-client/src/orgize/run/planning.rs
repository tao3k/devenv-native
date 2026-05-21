//! Agent-planning `orgize` command execution.

use anyhow::Result;
use xiuxian_wendao_parsers::{OrgizeAgentPlanningRequest, render_agent_planning};

use crate::orgize::OrgizeAgentPlanningArgs;
use crate::{ClientContext, CommandOutcome};

use super::paths::resolve_paths;

pub(super) fn run_agent_planning(
    args: &OrgizeAgentPlanningArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let rendered = render_agent_planning(&OrgizeAgentPlanningRequest {
        paths: resolve_paths(&args.paths, context),
        start_date: args.date.clone(),
        end_date: args.end.clone(),
        include_done: args.include_done,
        include_archived: args.include_archived,
        include_comments: args.include_comments,
        match_expression: args.match_expression.clone(),
    })?;
    print!("{rendered}");
    Ok(CommandOutcome::success())
}
