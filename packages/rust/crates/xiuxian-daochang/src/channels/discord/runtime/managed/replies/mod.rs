//! Discord managed reply branch for command-specific response rendering.

mod budget;
mod jobs;
mod memory;
mod mention;
mod partition;
mod session_context;
mod session_reply_format;

pub(super) use budget::{
    format_context_budget_not_found_json, format_context_budget_snapshot,
    format_context_budget_snapshot_json,
};
pub(super) use jobs::{
    format_job_metrics, format_job_metrics_json, format_job_not_found, format_job_not_found_json,
    format_job_status, format_job_status_json,
};
pub(super) use memory::{
    format_memory_recall_not_found, format_memory_recall_not_found_json,
    format_memory_recall_snapshot, format_memory_recall_snapshot_json,
};
pub(super) use mention::{
    format_session_mention_admin_required, format_session_mention_admin_required_json,
    format_session_mention_status, format_session_mention_status_json,
    format_session_mention_updated, format_session_mention_updated_json,
};
pub(super) use partition::{
    format_session_partition_admin_required, format_session_partition_admin_required_json,
    format_session_partition_error_json, format_session_partition_status,
    format_session_partition_status_json, format_session_partition_updated,
    format_session_partition_updated_json,
};
pub(super) use session_context::{
    format_session_context_snapshot, format_session_context_snapshot_json,
};
pub(super) use session_reply_format::{
    format_command_error_json, format_control_command_admin_required, format_session_feedback,
    format_session_feedback_json, format_session_feedback_unavailable_json,
    format_slash_command_permission_required, format_slash_help, format_slash_help_json,
};
