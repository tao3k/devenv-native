//! Orgize-backed document tooling for Wendao client surfaces.

mod agenda;
mod agent_tasks;
mod error;
mod format;
mod io;
mod lint;
mod sdd;
mod sparse_tree;

pub use agenda::{OrgizeAgentPlanningRequest, render_agent_planning};
pub use agent_tasks::{
    OrgizeAgentTaskProperty, OrgizeAgentTaskReadModelReport, OrgizeAgentTaskReadModelRequest,
    OrgizeAgentTaskRepeater, OrgizeAgentTaskRow, collect_agent_task_rows,
};
pub use error::OrgizeToolError;
pub use format::{OrgizeFormatReport, OrgizeFormatRequest, format_org_files};
pub use lint::{
    OrgizeLintFileReport, OrgizeLintOutputFormat, OrgizeLintRequest, OrgizeLintRunReport,
    lint_org_files,
};
pub use sdd::{
    OrgizeSddGraphDiffRequest, OrgizeSddStatusRequest, count_sdd_graph_drift,
    count_sdd_status_issues, render_sdd_graph_diff, render_sdd_status, render_sdd_status_json,
};
pub use sparse_tree::{
    OrgizeSparseTreeRenderOptions, OrgizeSparseTreeRequest, OrgizeSparseTreeVisibility,
    render_sparse_tree,
};
