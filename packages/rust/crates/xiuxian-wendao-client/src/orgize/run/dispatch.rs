//! Root `orgize` command dispatcher.

use anyhow::Result;

use crate::orgize::OrgizeCommand;
#[cfg(feature = "orgize-agent-read-model")]
use crate::orgize::read_model::{run_read_model, run_task_archive, run_task_list, run_task_report};
use crate::{ClientContext, CommandOutcome};

use super::basic::{run_format, run_lint};
use super::planning::run_agent_planning;
use super::sdd::run_sdd;
use super::sparse_tree::run_sparse_tree;

/// Run one Orgize-backed client command.
///
/// # Errors
///
/// Returns an error when the selected Orgize operation fails.
pub(crate) fn run_command(
    command: &OrgizeCommand,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    match command {
        OrgizeCommand::Fmt(args) => run_format(args, context),
        OrgizeCommand::Lint(args) => run_lint(args, context),
        OrgizeCommand::AgentPlanning(args) => run_agent_planning(args, context),
        #[cfg(feature = "orgize-agent-read-model")]
        OrgizeCommand::ReadModel(args) => run_read_model(args, context),
        #[cfg(feature = "orgize-agent-read-model")]
        OrgizeCommand::TaskList(args) => run_task_list(args, context),
        #[cfg(feature = "orgize-agent-read-model")]
        OrgizeCommand::TaskReport(args) => run_task_report(args, context),
        #[cfg(feature = "orgize-agent-read-model")]
        OrgizeCommand::TaskArchive(args) => run_task_archive(args, context),
        OrgizeCommand::SparseTree(args) => run_sparse_tree(args, context),
        OrgizeCommand::Sdd { command } => run_sdd(command, context),
    }
}
