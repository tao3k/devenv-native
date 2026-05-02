use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc;

use xiuxian_daochang::{Channel, ChannelMessage, TelegramSessionPartition};

use super::MockChannel;
use super::SessionIdentity;
use super::build_agent;
use super::build_job_manager;
use super::handle_inbound_message;
use super::partitioned_inbound_message;

pub(crate) async fn run_partition_reset_status_flow(
    partition: TelegramSessionPartition,
    reset_identity: SessionIdentity,
    status_identity: SessionIdentity,
    expect_shared_snapshot: bool,
) -> Result<()> {
    let reset_message = partitioned_inbound_message(partition, reset_identity, "/reset")?;
    let status_message = partitioned_inbound_message(partition, status_identity, "/resume status")?;

    let reset_session_id = format!("{}:{}", reset_message.channel, reset_message.session_key);
    let status_session_id = format!("{}:{}", status_message.channel, status_message.session_key);
    if expect_shared_snapshot {
        assert_eq!(
            reset_session_id, status_session_id,
            "expected shared partition to map into same logical session key"
        );
    } else {
        assert_ne!(
            reset_session_id, status_session_id,
            "expected isolated partition to map into different logical session keys"
        );
    }

    let agent = build_agent().await?;
    let channel = Arc::new(MockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel::<ChannelMessage>(8);

    agent
        .append_turn_for_session(&reset_session_id, "u1", "a1")
        .await?;
    agent
        .append_turn_for_session(&reset_session_id, "u2", "a2")
        .await?;

    assert!(
        handle_inbound_message(
            reset_message,
            &channel_dyn,
            &foreground_tx,
            &job_manager,
            &agent,
        )
        .await
    );
    assert!(
        handle_inbound_message(
            status_message,
            &channel_dyn,
            &foreground_tx,
            &job_manager,
            &agent,
        )
        .await
    );

    assert!(
        foreground_rx.try_recv().is_err(),
        "session commands should not enter foreground queue"
    );
    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 2);
    assert!(sent[0].0.contains("Session context reset."));
    assert!(sent[0].0.contains("messages_cleared=4"));
    if expect_shared_snapshot {
        assert!(sent[1].0.contains("Saved session context snapshot:"));
        assert!(sent[1].0.contains("messages=4"));
    } else {
        assert!(
            sent[1]
                .0
                .contains("No saved session context snapshot found.")
        );
    }
    Ok(())
}
