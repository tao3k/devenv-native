//! Root `orgize` subcommand enum.

use clap::Subcommand;

/// Orgize-backed client subcommands.
#[derive(Subcommand, Debug)]
pub enum OrgizeCommand {
    /// Format Org files with the upstream Orgize formatter.
    Fmt(super::OrgizeFormatArgs),
    /// Lint Org files with the upstream Orgize linter.
    Lint(super::OrgizeLintArgs),
    /// Work with Org Babel eval contracts without executing code.
    Eval {
        /// Eval-oriented subcommand selection.
        #[command(subcommand)]
        command: super::OrgizeEvalCommand,
    },
    /// Render agent planning cards from Org agenda syntax.
    AgentPlanning(super::OrgizeAgentPlanningArgs),
    /// Materialize the default `DuckDB` read model for agent Org tasks.
    #[cfg(feature = "orgize-agent-read-model")]
    ReadModel(super::OrgizeReadModelArgs),
    /// Refresh the `DuckDB` read model and list agent Org task rows.
    #[cfg(feature = "orgize-agent-read-model")]
    TaskList(super::OrgizeTaskListArgs),
    /// Probe compact agent Org task candidates from remembered text.
    #[cfg(feature = "orgize-agent-read-model")]
    TaskProbe(super::OrgizeTaskProbeArgs),
    /// Show one agent Org task subtree by stable Org section ID.
    #[cfg(feature = "orgize-agent-read-model")]
    OgridShow(super::OrgizeOgridShowArgs),
    /// Show one agent Org task's SDD/plan relation graph.
    #[cfg(feature = "orgize-agent-read-model")]
    TaskSdd(super::OrgizeTaskSddArgs),
    /// Render recent agent Org task recovery candidates.
    #[cfg(feature = "orgize-agent-read-model")]
    TaskRecover(super::OrgizeTaskRecoverArgs),
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
