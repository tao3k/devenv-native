use std::sync::Arc;

use anyhow::Result;

use xiuxian_daochang::Channel;

use super::support::{
    MockChannel, build_agent, inbound, process_discord_message, start_job_manager,
};

#[tokio::test]
async fn process_discord_message_handles_background_submit_ack() -> Result<()> {
    let agent = build_agent().await?;
    let job_manager = start_job_manager(&agent);
    let channel = Arc::new(MockChannel::with_acl(true, std::iter::empty::<&str>()));
    let channel_dyn: Arc<dyn Channel> = channel.clone();

    process_discord_message(
        agent,
        channel_dyn,
        inbound("/bg collect incident summary"),
        &job_manager,
        10,
    )
    .await;

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert!(sent[0].0.contains("Queued background job `job-"));
    assert!(sent[0].0.contains("Use `/job "));
    Ok(())
}

#[tokio::test]
async fn process_discord_message_session_memory_includes_gate_policy_in_text() -> Result<()> {
    let agent = build_agent().await?;
    let job_manager = start_job_manager(&agent);
    let channel = Arc::new(MockChannel::with_acl(true, std::iter::empty::<&str>()));
    let channel_dyn: Arc<dyn Channel> = channel.clone();

    process_discord_message(
        agent,
        channel_dyn,
        inbound("/session memory"),
        &job_manager,
        10,
    )
    .await;

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert!(
        sent[0]
            .0
            .contains("- Session scope: `discord:3001:2001:1001`")
    );
    assert!(sent[0].0.contains("- `memory_enabled=false`"));
    assert!(
        sent[0]
            .0
            .contains("- `configured_backend=-` / `active_backend=-`")
    );
    Ok(())
}

#[tokio::test]
async fn process_discord_message_session_memory_json_includes_gate_policy_fields() -> Result<()> {
    let agent = build_agent().await?;
    let job_manager = start_job_manager(&agent);
    let channel = Arc::new(MockChannel::with_acl(true, std::iter::empty::<&str>()));
    let channel_dyn: Arc<dyn Channel> = channel.clone();

    process_discord_message(
        agent,
        channel_dyn,
        inbound("/session memory json"),
        &job_manager,
        10,
    )
    .await;

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 1);
    let payload: serde_json::Value = serde_json::from_str(&sent[0].0)?;
    assert_eq!(payload["kind"], "session_memory");
    assert_eq!(payload["session_scope"], "discord:3001:2001:1001");
    assert!(payload["runtime"]["gate_promote_threshold"].is_null());
    assert!(payload["runtime"]["gate_obsolete_threshold"].is_null());
    assert!(payload["runtime"]["gate_promote_min_usage"].is_null());
    assert!(payload["runtime"]["gate_obsolete_min_usage"].is_null());
    assert_eq!(payload["metrics"]["embedding_success_total"], 0);
    assert_eq!(payload["metrics"]["embedding_timeout_total"], 0);
    assert_eq!(payload["metrics"]["embedding_cooldown_reject_total"], 0);
    assert_eq!(payload["metrics"]["embedding_unavailable_total"], 0);
    Ok(())
}
