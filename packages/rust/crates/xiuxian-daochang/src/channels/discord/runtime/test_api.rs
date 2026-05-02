//! Test-facing Discord runtime helper methods.

use std::sync::Arc;

use crate::agent::Agent;
use crate::channels::managed_runtime::ForegroundQueueMode;
use crate::channels::{Channel, ChannelMessage};
use crate::jobs::{JobCompletion, JobManager};

use super::{ForegroundInterruptController, dispatch, managed, telemetry};

pub(crate) async fn test_process_discord_message(
    agent: Arc<Agent>,
    channel: Arc<dyn Channel>,
    msg: ChannelMessage,
    job_manager: &Arc<JobManager>,
    turn_timeout_secs: u64,
) {
    dispatch::process_discord_message(agent, channel, msg, job_manager, turn_timeout_secs).await;
}

pub(crate) async fn test_process_discord_message_with_interrupt(
    agent: Arc<Agent>,
    channel: Arc<dyn Channel>,
    msg: ChannelMessage,
    job_manager: &Arc<JobManager>,
    turn_timeout_secs: u64,
    foreground_queue_mode: ForegroundQueueMode,
    interrupt_controller: &ForegroundInterruptController,
) {
    dispatch::process_discord_message_with_interrupt(
        agent,
        channel,
        msg,
        job_manager,
        turn_timeout_secs,
        foreground_queue_mode,
        interrupt_controller,
    )
    .await;
}

pub(crate) fn test_resolve_snapshot_interval_secs<F>(lookup: F) -> Option<u64>
where
    F: Fn(&str) -> Option<String>,
{
    telemetry::resolve_snapshot_interval_secs(lookup)
}

pub(crate) fn test_interrupted_reply_is_suppressed(
    msg: &ChannelMessage,
    turn_timeout_secs: u64,
) -> bool {
    dispatch::test_interrupted_reply_is_suppressed(msg, turn_timeout_secs)
}

pub(crate) async fn test_push_background_completion(
    channel: &Arc<dyn Channel>,
    agent: &Arc<Agent>,
    completion: JobCompletion,
) {
    managed::push_background_completion(channel, agent, completion).await;
}
