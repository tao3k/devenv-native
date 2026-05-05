//! Test coverage for xiuxian-daochang behavior.

use super::{
    MockChannel, build_agent, build_job_manager, handle_inbound_message_with_interrupt, inbound,
};
use anyhow::{Result, anyhow};
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc;
use xiuxian_daochang::test_support::TelegramForegroundInterruptController as ForegroundInterruptController;
use xiuxian_daochang::{Channel, ForegroundQueueMode};

#[tokio::test]
async fn runtime_handle_inbound_plain_message_preempts_active_session_generation() -> Result<()> {
    let agent = build_agent().await?;
    let channel = Arc::new(MockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel(8);
    let interrupt_controller = ForegroundInterruptController::default();
    let message = inbound("replace the prior answer with a concise summary");
    let expected_content = message.content.clone();
    let session_id = format!("{}:{}", message.channel, message.session_key);
    let mut interrupt_rx = interrupt_controller.begin_generation(&session_id);
    let initial_generation = *interrupt_rx.borrow();
    let handled = handle_inbound_message_with_interrupt(
        message,
        &channel_dyn,
        &foreground_tx,
        &interrupt_controller,
        &job_manager,
        &agent,
        ForegroundQueueMode::Interrupt,
    )
    .await;
    assert!(handled, "plain message should be accepted by runtime");
    let queued = foreground_rx
        .recv()
        .await
        .ok_or_else(|| anyhow!("plain message should enter foreground queue"))?;
    assert_eq!(queued.content, expected_content);
    tokio::time::timeout(Duration::from_millis(200), interrupt_rx.changed()).await??;
    assert!(
        *interrupt_rx.borrow() > initial_generation,
        "new plain message should preempt active generation in same session"
    );
    assert!(
        channel.sent_messages().await.is_empty(),
        "preemption should not emit immediate control reply"
    );
    interrupt_controller.end_generation(&session_id);
    Ok(())
}

#[tokio::test]
async fn runtime_handle_inbound_plain_message_does_not_preempt_other_session() -> Result<()> {
    let agent = build_agent().await?;
    let channel = Arc::new(MockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel(8);
    let interrupt_controller = ForegroundInterruptController::default();
    let message = inbound("continue");
    let other_session_id = "telegram:-200:999";
    let mut interrupt_rx = interrupt_controller.begin_generation(other_session_id);
    let initial_generation = *interrupt_rx.borrow();
    let handled = handle_inbound_message_with_interrupt(
        message,
        &channel_dyn,
        &foreground_tx,
        &interrupt_controller,
        &job_manager,
        &agent,
        ForegroundQueueMode::Interrupt,
    )
    .await;
    assert!(handled, "plain message should be accepted by runtime");
    assert!(
        foreground_rx.recv().await.is_some(),
        "plain message should enter foreground queue"
    );
    let changed = tokio::time::timeout(Duration::from_millis(120), interrupt_rx.changed()).await;
    assert!(
        changed.is_err(),
        "message from different session must not preempt active generation"
    );
    assert_eq!(
        *interrupt_rx.borrow(),
        initial_generation,
        "interrupt generation should stay unchanged for other sessions"
    );
    assert!(
        channel.sent_messages().await.is_empty(),
        "cross-session no-op should not emit control reply"
    );
    interrupt_controller.end_generation(other_session_id);
    Ok(())
}

#[tokio::test]
async fn runtime_handle_inbound_plain_message_queues_without_preempt_when_mode_is_queue()
-> Result<()> {
    let agent = build_agent().await?;
    let channel = Arc::new(MockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel(8);
    let interrupt_controller = ForegroundInterruptController::default();
    let message = inbound("continue");
    let expected_content = message.content.clone();
    let session_id = format!("{}:{}", message.channel, message.session_key);
    let mut interrupt_rx = interrupt_controller.begin_generation(&session_id);
    let initial_generation = *interrupt_rx.borrow();

    let handled = handle_inbound_message_with_interrupt(
        message,
        &channel_dyn,
        &foreground_tx,
        &interrupt_controller,
        &job_manager,
        &agent,
        ForegroundQueueMode::Queue,
    )
    .await;
    assert!(handled, "plain message should be accepted by runtime");
    let queued = foreground_rx
        .recv()
        .await
        .ok_or_else(|| anyhow!("plain message should enter foreground queue"))?;
    assert_eq!(queued.content, expected_content);
    let changed = tokio::time::timeout(Duration::from_millis(120), interrupt_rx.changed()).await;
    assert!(
        changed.is_err(),
        "queue mode should not preempt active generation"
    );
    assert_eq!(
        *interrupt_rx.borrow(),
        initial_generation,
        "interrupt generation must remain unchanged when queue mode is active"
    );
    interrupt_controller.end_generation(&session_id);
    Ok(())
}
