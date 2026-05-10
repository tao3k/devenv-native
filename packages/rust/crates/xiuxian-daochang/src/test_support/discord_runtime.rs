//! Discord runtime helpers exposed for integration tests.

use std::sync::Arc;

use crate::channels::discord::runtime;
use crate::jobs::{JobCompletion, JobManager};
use crate::{Agent, Channel, ChannelMessage, ForegroundQueueMode};

/// Public wrapper for Discord foreground interrupt coordination in tests.
#[derive(Clone, Default)]
pub struct DiscordForegroundInterruptController {
    inner: runtime::ForegroundInterruptController,
}

impl DiscordForegroundInterruptController {
    #[must_use]
    /// Begin one foreground generation stream for a logical session.
    pub fn begin_generation(
        &self,
        session_id: impl AsRef<str>,
    ) -> tokio::sync::watch::Receiver<u64> {
        self.inner.begin_generation(session_id.as_ref())
    }

    /// Mark one foreground generation as completed for a logical session.
    pub fn end_generation(&self, session_id: impl AsRef<str>) {
        self.inner.end_generation(session_id.as_ref());
    }
}

/// Test-only harness for driving Discord foreground runtime scheduling.
pub struct DiscordForegroundRuntimeHarness {
    inner: runtime::TestDiscordForegroundRuntime,
    completion_rx: tokio::sync::mpsc::Receiver<JobCompletion>,
}

impl DiscordForegroundRuntimeHarness {
    /// Submit one inbound foreground turn to the Discord runtime.
    pub async fn spawn_foreground_turn(&mut self, msg: ChannelMessage) {
        self.inner.spawn_foreground_turn(msg).await;
    }

    /// Whether there are still foreground tasks owned by the runtime.
    #[must_use]
    pub fn has_foreground_tasks(&self) -> bool {
        self.inner.has_foreground_tasks()
    }

    /// Wait for the next foreground task to complete.
    pub async fn join_next_foreground_task(&mut self) {
        self.inner.join_next_foreground_task().await;
    }

    /// Abort and drain all remaining foreground tasks.
    pub async fn abort_and_drain_foreground_tasks(&mut self) {
        self.inner.abort_and_drain_foreground_tasks().await;
    }

    /// Receive and forward the next background completion through the runtime.
    pub async fn push_next_background_completion(&mut self) -> bool {
        match self.completion_rx.recv().await {
            Some(completion) => {
                self.inner.push_completion(completion).await;
                true
            }
            None => false,
        }
    }
}

/// Build a test-only Discord foreground runtime harness.
#[must_use]
pub fn build_discord_foreground_runtime(
    agent: Arc<Agent>,
    channel: Arc<dyn Channel>,
    turn_timeout_secs: u64,
    foreground_max_in_flight_messages: usize,
    foreground_queue_mode: ForegroundQueueMode,
) -> DiscordForegroundRuntimeHarness {
    let (inner, completion_rx) = runtime::test_build_discord_foreground_runtime(
        agent,
        channel,
        turn_timeout_secs,
        foreground_max_in_flight_messages,
        foreground_queue_mode,
    );
    DiscordForegroundRuntimeHarness {
        inner,
        completion_rx,
    }
}

/// Processes one Discord foreground message.
pub async fn process_discord_message(
    agent: Arc<Agent>,
    channel: Arc<dyn Channel>,
    msg: ChannelMessage,
    job_manager: &Arc<JobManager>,
    turn_timeout_secs: u64,
) {
    runtime::test_process_discord_message(agent, channel, msg, job_manager, turn_timeout_secs)
        .await;
}

/// Processes one Discord foreground message with interrupt coordination.
pub async fn process_discord_message_with_interrupt(
    agent: Arc<Agent>,
    channel: Arc<dyn Channel>,
    msg: ChannelMessage,
    job_manager: &Arc<JobManager>,
    turn_timeout_secs: u64,
    foreground_queue_mode: ForegroundQueueMode,
    interrupt_controller: &DiscordForegroundInterruptController,
) {
    runtime::test_process_discord_message_with_interrupt(
        agent,
        channel,
        msg,
        job_manager,
        turn_timeout_secs,
        foreground_queue_mode,
        &interrupt_controller.inner,
    )
    .await;
}

/// Pushes a background job completion into the Discord runtime surface.
pub async fn push_discord_background_completion(
    channel: &Arc<dyn Channel>,
    agent: &Arc<Agent>,
    completion: JobCompletion,
) {
    runtime::test_push_background_completion(channel, agent, completion).await;
}

#[must_use]
/// Resolves the Discord runtime snapshot interval from a lookup function.
pub fn resolve_discord_snapshot_interval_secs<F>(lookup: F) -> Option<u64>
where
    F: Fn(&str) -> Option<String>,
{
    runtime::test_resolve_snapshot_interval_secs(lookup)
}

#[must_use]
/// Check whether an interrupted foreground turn should suppress user-visible replies.
pub fn discord_interrupted_reply_is_suppressed(
    msg: &ChannelMessage,
    turn_timeout_secs: u64,
) -> bool {
    runtime::test_interrupted_reply_is_suppressed(msg, turn_timeout_secs)
}
