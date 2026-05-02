use std::sync::Arc;

use crate::channels::managed_commands::SLASH_SCOPE_JOBS_SUMMARY as TELEGRAM_SLASH_SCOPE_JOBS_SUMMARY;
use crate::channels::telegram::commands::parse_jobs_summary_command;
use crate::channels::traits::{Channel, ChannelMessage};
use crate::jobs::JobManager;

use super::{
    EVENT_TELEGRAM_COMMAND_JOBS_SUMMARY_JSON_REPLIED, EVENT_TELEGRAM_COMMAND_JOBS_SUMMARY_REPLIED,
};
use crate::channels::telegram::runtime::jobs::command_handlers::slash_acl::ensure_slash_command_authorized;
use crate::channels::telegram::runtime::jobs::observability::send_with_observability;
use crate::channels::telegram::runtime::jobs::replies::{
    format_job_metrics, format_job_metrics_json,
};

pub(in crate::channels::telegram::runtime::jobs) async fn try_handle_jobs_summary_command(
    msg: &ChannelMessage,
    channel: &Arc<dyn Channel>,
    job_manager: &Arc<JobManager>,
) -> bool {
    let Some(format) = parse_jobs_summary_command(&msg.content) else {
        return false;
    };

    if !ensure_slash_command_authorized(channel, msg, TELEGRAM_SLASH_SCOPE_JOBS_SUMMARY, "/jobs")
        .await
    {
        return true;
    }

    let command_event = if format.is_json() {
        EVENT_TELEGRAM_COMMAND_JOBS_SUMMARY_JSON_REPLIED
    } else {
        EVENT_TELEGRAM_COMMAND_JOBS_SUMMARY_REPLIED
    };
    let metrics = job_manager.metrics().await;
    let metrics_msg = if format.is_json() {
        format_job_metrics_json(&metrics)
    } else {
        format_job_metrics(&metrics)
    };
    send_with_observability(
        channel,
        &metrics_msg,
        &msg.recipient,
        "Failed to send job metrics",
        Some(command_event),
        Some(&msg.session_key),
    )
    .await;
    true
}
