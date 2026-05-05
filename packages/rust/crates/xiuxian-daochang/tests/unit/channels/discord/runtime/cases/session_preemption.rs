//! Test coverage for xiuxian-daochang behavior.

use super::support::{
    ForegroundInterruptController, MockChannel, build_agent, inbound,
    process_discord_message_with_interrupt, start_job_manager,
};
use anyhow::Result;
use std::{sync::Arc, time::Duration};
use tokio::time::timeout;
use xiuxian_daochang::test_support::discord_interrupted_reply_is_suppressed;
use xiuxian_daochang::{Channel, ForegroundQueueMode};

macro_rules! dispatch_message {
    ($agent:expr, $channel:expr, $message:expr, $job_manager:expr, $queue_mode:expr, $interrupt:expr) => {
        process_discord_message_with_interrupt(
            $agent,
            $channel,
            $message,
            $job_manager,
            1,
            $queue_mode,
            $interrupt,
        )
        .await;
    };
}

#[tokio::test]
async fn process_discord_message_preempts_same_session_foreground_generation() -> Result<()> {
    let agent = build_agent().await?;
    let job_manager = start_job_manager(&agent);
    let channel = Arc::new(MockChannel::with_acl(true, std::iter::empty::<&str>()));
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let interrupt_controller = ForegroundInterruptController::default();
    let msg = inbound("replace the previous answer with a concise summary");
    let session_id = format!("{}:{}", msg.channel, msg.session_key);
    let mut interrupt_rx = interrupt_controller.begin_generation(&session_id);
    let initial_generation = *interrupt_rx.borrow();
    dispatch_message!(
        agent,
        channel_dyn,
        msg,
        &job_manager,
        ForegroundQueueMode::Interrupt,
        &interrupt_controller
    );
    timeout(Duration::from_millis(200), interrupt_rx.changed()).await??;
    assert!(
        *interrupt_rx.borrow() > initial_generation,
        "same-session plain message should preempt active generation"
    );
    interrupt_controller.end_generation(&session_id);
    Ok(())
}

#[tokio::test]
async fn process_discord_message_does_not_preempt_other_session_generation() -> Result<()> {
    let agent = build_agent().await?;
    let job_manager = start_job_manager(&agent);
    let channel = Arc::new(MockChannel::with_acl(true, std::iter::empty::<&str>()));
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let interrupt_controller = ForegroundInterruptController::default();
    let msg = inbound("continue");
    let other_session_id = "discord:other:session";
    let mut interrupt_rx = interrupt_controller.begin_generation(other_session_id);
    let initial_generation = *interrupt_rx.borrow();
    dispatch_message!(
        agent,
        channel_dyn,
        msg,
        &job_manager,
        ForegroundQueueMode::Interrupt,
        &interrupt_controller
    );
    let changed = timeout(Duration::from_millis(120), interrupt_rx.changed()).await;
    assert!(
        changed.is_err(),
        "different-session message must not preempt active generation"
    );
    assert_eq!(
        *interrupt_rx.borrow(),
        initial_generation,
        "interrupt generation must remain unchanged for different sessions"
    );
    interrupt_controller.end_generation(other_session_id);
    Ok(())
}

#[tokio::test]
async fn process_discord_message_stop_command_interrupts_active_generation() -> Result<()> {
    let agent = build_agent().await?;
    let job_manager = start_job_manager(&agent);
    let channel = Arc::new(MockChannel::with_acl(true, std::iter::empty::<&str>()));
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let interrupt_controller = ForegroundInterruptController::default();
    let msg = inbound("/stop");
    let session_id = format!("{}:{}", msg.channel, msg.session_key);
    let mut interrupt_rx = interrupt_controller.begin_generation(&session_id);
    let initial_generation = *interrupt_rx.borrow();
    dispatch_message!(
        agent,
        channel_dyn,
        msg,
        &job_manager,
        ForegroundQueueMode::Queue,
        &interrupt_controller
    );
    timeout(Duration::from_millis(200), interrupt_rx.changed()).await??;
    assert!(
        *interrupt_rx.borrow() > initial_generation,
        "stop command should preempt the active generation"
    );
    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert!(sent[0].0.contains("Stop signal sent."));
    interrupt_controller.end_generation(&session_id);
    Ok(())
}

#[tokio::test]
async fn process_discord_message_queue_mode_does_not_preempt_same_session_generation() -> Result<()>
{
    let agent = build_agent().await?;
    let job_manager = start_job_manager(&agent);
    let channel = Arc::new(MockChannel::with_acl(true, std::iter::empty::<&str>()));
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let interrupt_controller = ForegroundInterruptController::default();
    let msg = inbound("continue");
    let session_id = format!("{}:{}", msg.channel, msg.session_key);
    let mut interrupt_rx = interrupt_controller.begin_generation(&session_id);
    let initial_generation = *interrupt_rx.borrow();
    dispatch_message!(
        agent,
        channel_dyn,
        msg,
        &job_manager,
        ForegroundQueueMode::Queue,
        &interrupt_controller
    );
    let changed = timeout(Duration::from_millis(120), interrupt_rx.changed()).await;
    assert!(
        changed.is_err(),
        "queue mode should not preempt active foreground generation"
    );
    assert_eq!(
        *interrupt_rx.borrow(),
        initial_generation,
        "interrupt generation must remain unchanged in queue mode"
    );
    interrupt_controller.end_generation(&session_id);
    Ok(())
}

#[test]
fn interrupted_foreground_turn_reply_is_suppressed() {
    let msg = inbound("continue");
    assert!(
        discord_interrupted_reply_is_suppressed(&msg, 30),
        "discord interrupted turns should not emit a user-visible interruption reply"
    );
}
