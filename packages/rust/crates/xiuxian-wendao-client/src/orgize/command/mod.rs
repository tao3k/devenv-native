//! Command-line model for Orgize-backed client operations.

mod basic;
mod eval;
mod planning;
#[cfg(feature = "orgize-agent-read-model")]
mod read_model;
mod root;
mod sdd;
mod sparse_tree;

pub use basic::{OrgizeFormatArgs, OrgizeLintArgs, OrgizeLintFormatArg};
pub use eval::OrgizeEvalCommand;
pub(crate) use eval::{OrgizeEvalPatchArgs, OrgizeEvalPlanArgs};
pub use planning::OrgizeAgentPlanningArgs;
#[cfg(feature = "orgize-agent-read-model")]
pub use read_model::{
    OrgizeOgridShowArgs, OrgizeReadModelArgs, OrgizeTaskArchiveArgs, OrgizeTaskListArgs,
    OrgizeTaskListView, OrgizeTaskProbeArgs, OrgizeTaskRecoverArgs, OrgizeTaskReportArgs,
    OrgizeTaskSddArgs,
};
pub use root::OrgizeCommand;
pub use sdd::{OrgizeSddCommand, OrgizeSddGraphDiffArgs, OrgizeSddStatusArgs};
pub use sparse_tree::OrgizeSparseTreeArgs;
