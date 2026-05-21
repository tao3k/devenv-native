//! Root `orgize` subcommand enum.

use clap::Subcommand;

/// Orgize-backed client subcommands.
#[derive(Subcommand, Debug)]
pub enum OrgizeCommand {
    /// Format Org files with the upstream Orgize formatter.
    Fmt(super::OrgizeFormatArgs),
    /// Lint Org files with the upstream Orgize linter.
    Lint(super::OrgizeLintArgs),
    /// Render agent planning cards from Org agenda syntax.
    AgentPlanning(super::OrgizeAgentPlanningArgs),
    /// Materialize the default `DuckDB` read model for agent Org tasks.
    #[cfg(feature = "orgize-agent-read-model")]
    ReadModel(super::OrgizeReadModelArgs),
    /// Refresh the `DuckDB` read model and list agent Org task rows.
    #[cfg(feature = "orgize-agent-read-model")]
    TaskList(super::OrgizeTaskListArgs),
    /// Refresh the `DuckDB` read model and summarize agent Org task rows.
    #[cfg(feature = "orgize-agent-read-model")]
    TaskReport(super::OrgizeTaskReportArgs),
    /// Plan or apply archival for completed agent Org task rows.
    #[cfg(feature = "orgize-agent-read-model")]
    TaskArchive(super::OrgizeTaskArchiveArgs),
    /// Render sparse-tree cards from Org match/text predicates.
    SparseTree(super::OrgizeSparseTreeArgs),
    /// Work with Org-native SDD projections.
    Sdd {
        /// SDD-oriented subcommand selection.
        #[command(subcommand)]
        command: super::OrgizeSddCommand,
    },
}
