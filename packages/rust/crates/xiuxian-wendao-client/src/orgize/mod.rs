//! Orgize-backed client command surface.

mod command;
#[cfg(feature = "orgize-agent-read-model")]
mod read_model;
mod run;

pub use command::{
    OrgizeAgentPlanningArgs, OrgizeCommand, OrgizeEvalCommand, OrgizeFormatArgs, OrgizeLintArgs,
    OrgizeLintFormatArg, OrgizeSddCommand, OrgizeSddGraphDiffArgs, OrgizeSddStatusArgs,
    OrgizeSparseTreeArgs,
};
#[cfg(feature = "orgize-agent-read-model")]
pub use command::{
    OrgizeOrgidShowArgs, OrgizeReadModelArgs, OrgizeTaskArchiveArgs, OrgizeTaskListArgs,
    OrgizeTaskListView, OrgizeTaskProbeArgs, OrgizeTaskRecoverArgs, OrgizeTaskReportArgs,
    OrgizeTaskSddArgs,
};
#[cfg(all(feature = "performance", feature = "orgize-agent-read-model"))]
pub use read_model::perf_support;
pub(crate) use run::run_command;
