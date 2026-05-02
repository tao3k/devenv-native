//! Test-facing Telegram runtime helpers.

use std::sync::Arc;

use tokio::sync::mpsc;

use crate::agent::Agent;
use crate::channels::managed_runtime::ForegroundQueueMode;
use crate::channels::{Channel, ChannelMessage};
use crate::jobs::JobManager;

use super::dispatch::ForegroundInterruptController;
use super::{jobs, telemetry};

pub(crate) async fn test_handle_inbound_message_with_interrupt(
    msg: ChannelMessage,
    channel: &Arc<dyn Channel>,
    foreground_tx: &mpsc::Sender<ChannelMessage>,
    interrupt_controller: &ForegroundInterruptController,
    job_manager: &Arc<JobManager>,
    agent: &Arc<Agent>,
    queue_mode: ForegroundQueueMode,
) -> bool {
    jobs::handle_inbound_message_with_interrupt(
        msg,
        channel,
        foreground_tx,
        interrupt_controller,
        job_manager,
        agent,
        queue_mode,
    )
    .await
}

pub(crate) async fn test_push_background_completion(
    channel: &Arc<dyn Channel>,
    agent: &Arc<Agent>,
    completion: crate::jobs::JobCompletion,
) {
    jobs::push_background_completion(channel, agent, completion).await;
}

pub(crate) fn test_resolve_snapshot_interval_secs<F>(lookup: F) -> Option<u64>
where
    F: Fn(&str) -> Option<String>,
{
    telemetry::resolve_snapshot_interval_secs(lookup)
}

#[must_use]
pub(crate) fn test_log_preview(s: &str) -> String {
    jobs::log_preview(s)
}
