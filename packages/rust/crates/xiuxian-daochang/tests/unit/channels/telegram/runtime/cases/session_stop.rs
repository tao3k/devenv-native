//! Test coverage for xiuxian-daochang behavior.

use std::{sync::Arc, time::Duration};

use anyhow::Result;
use tokio::sync::mpsc;

use xiuxian_daochang::test_support::TelegramForegroundInterruptController as ForegroundInterruptController;
use xiuxian_daochang::{Channel, ForegroundQueueMode};

use super::{
    MockChannel, build_agent, build_job_manager, handle_inbound_message,
    handle_inbound_message_with_interrupt, inbound,
};

#[tokio::test]
async fn runtime_handle_inbound_stop_without_active_turn_reports_idle() -> Result<()> {
    let agent = build_agent().await?;
    let channel = Arc::new(MockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel(8);

    let handled = handle_inbound_message(
        inbound("/stop"),
        &channel_dyn,
        &foreground_tx,
        &job_manager,
        &agent,
    )
    .await;
    assert!(handled, "stop command should be handled by runtime");
    assert!(
        foreground_rx.try_recv().is_err(),
        "stop command should not enter foreground queue"
    );

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert!(
        sent[0]
            .0
            .contains("No active foreground generation to stop in this session."),
        "stop idle response should be explicit"
    );

    Ok(())
}

#[tokio::test]
async fn runtime_handle_inbound_stop_interrupts_active_session_generation() -> Result<()> {
    let agent = build_agent().await?;
    let channel = Arc::new(MockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel(8);
    let interrupt_controller = ForegroundInterruptController::default();

    let message = inbound("/stop");
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
    assert!(handled, "stop command should be handled by runtime");
    assert!(
        foreground_rx.try_recv().is_err(),
        "stop command should not enter foreground queue"
    );

    tokio::time::timeout(Duration::from_millis(200), interrupt_rx.changed()).await??;
    assert!(
        *interrupt_rx.borrow() > initial_generation,
        "interrupt generation should advance after stop command"
    );

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert!(
        sent[0]
            .0
            .contains("Stop signal sent. Current foreground generation is being interrupted."),
        "stop response should confirm interrupt dispatch"
    );
    interrupt_controller.end_generation(&session_id);

    Ok(())
}
