//! Orgize-backed client command surface.

mod command;
#[cfg(feature = "orgize-agent-read-model")]
mod read_model;
mod run;

pub use command::{
    OrgizeAgentPlanningArgs, OrgizeCommand, OrgizeFormatArgs, OrgizeLintArgs, OrgizeLintFormatArg,
    OrgizeSddCommand, OrgizeSddGraphDiffArgs, OrgizeSddStatusArgs, OrgizeSparseTreeArgs,
};
#[cfg(feature = "orgize-agent-read-model")]
pub use command::{
    OrgizeReadModelArgs, OrgizeTaskArchiveArgs, OrgizeTaskListArgs, OrgizeTaskListView,
    OrgizeTaskReportArgs,
};
#[cfg(all(feature = "performance", feature = "orgize-agent-read-model"))]
pub use read_model::perf_support;
pub(crate) use run::run_command;
