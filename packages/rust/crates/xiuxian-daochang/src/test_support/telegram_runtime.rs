//! Telegram runtime helpers exposed for integration tests.

use std::sync::Arc;

use tokio::sync::mpsc;

use crate::channels::telegram::runtime;
use crate::jobs::JobCompletion;
use crate::{Agent, Channel, ChannelMessage, ForegroundQueueMode, JobManager};

/// Public wrapper for Telegram foreground interrupt coordination in tests.
#[derive(Clone, Default)]
pub struct TelegramForegroundInterruptController {
    inner: runtime::ForegroundInterruptController,
}

impl TelegramForegroundInterruptController {
    /// Begin one foreground generation stream for a logical session.
    #[must_use]
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

/// Handles one Telegram inbound message with interrupt coordination.
pub async fn handle_telegram_inbound_message_with_interrupt(
    msg: ChannelMessage,
    channel: &Arc<dyn Channel>,
    foreground_tx: &mpsc::Sender<ChannelMessage>,
    interrupt_controller: &TelegramForegroundInterruptController,
    job_manager: &Arc<JobManager>,
    agent: &Arc<Agent>,
    queue_mode: ForegroundQueueMode,
) -> bool {
    runtime::test_handle_inbound_message_with_interrupt(
        msg,
        channel,
        foreground_tx,
        &interrupt_controller.inner,
        job_manager,
        agent,
        queue_mode,
    )
    .await
}

/// Pushes a background job completion into the Telegram runtime surface.
pub async fn push_telegram_background_completion(
    channel: &Arc<dyn Channel>,
    agent: &Arc<Agent>,
    completion: JobCompletion,
) {
    runtime::test_push_background_completion(channel, agent, completion).await;
}

#[must_use]
/// Resolves the Telegram runtime snapshot interval from a lookup function.
pub fn resolve_telegram_snapshot_interval_secs<F>(lookup: F) -> Option<u64>
where
    F: Fn(&str) -> Option<String>,
{
    runtime::test_resolve_snapshot_interval_secs(lookup)
}

#[must_use]
/// Builds the bounded preview used in Telegram runtime logs.
pub fn telegram_log_preview(s: &str) -> String {
    runtime::test_log_preview(s)
}
