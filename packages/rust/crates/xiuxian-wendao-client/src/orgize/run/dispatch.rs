//! Root `orgize` command dispatcher.

use anyhow::Result;

use crate::orgize::OrgizeCommand;
#[cfg(feature = "orgize-agent-read-model")]
use crate::orgize::read_model::{
    run_orgid_show, run_read_model, run_task_archive, run_task_list, run_task_probe,
    run_task_recover, run_task_report, run_task_sdd,
};
use crate::{ClientContext, CommandOutcome};

use super::basic::{run_format, run_lint};
use super::eval::run_eval;
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
        OrgizeCommand::Eval { command } => run_eval(command, context),
        OrgizeCommand::AgentPlanning(args) => run_agent_planning(args, context),
        #[cfg(feature = "orgize-agent-read-model")]
        OrgizeCommand::ReadModel(args) => run_read_model(args, context),
        #[cfg(feature = "orgize-agent-read-model")]
        OrgizeCommand::TaskList(args) => run_task_list(args, context),
        #[cfg(feature = "orgize-agent-read-model")]
        OrgizeCommand::TaskProbe(args) => run_task_probe(args, context),
        #[cfg(feature = "orgize-agent-read-model")]
        OrgizeCommand::OrgidShow(args) => run_orgid_show(args, context),
        #[cfg(feature = "orgize-agent-read-model")]
        OrgizeCommand::TaskSdd(args) => run_task_sdd(args, context),
        #[cfg(feature = "orgize-agent-read-model")]
        OrgizeCommand::TaskRecover(args) => run_task_recover(args, context),
        #[cfg(feature = "orgize-agent-read-model")]
        OrgizeCommand::TaskReport(args) => run_task_report(args, context),
        #[cfg(feature = "orgize-agent-read-model")]
        OrgizeCommand::TaskArchive(args) => run_task_archive(args, context),
        OrgizeCommand::SparseTree(args) => run_sparse_tree(args, context),
        OrgizeCommand::Sdd { command } => run_sdd(command, context),
    }
}
