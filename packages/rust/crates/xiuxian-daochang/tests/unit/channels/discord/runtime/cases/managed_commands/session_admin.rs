use std::sync::Arc;

use anyhow::Result;

use xiuxian_daochang::Channel;

use super::support::{
    MockChannel, build_agent, inbound, process_discord_message, start_job_manager,
};

#[tokio::test]
async fn process_discord_message_handles_session_admin_set_and_status_json() -> Result<()> {
    let agent = build_agent().await?;
    let job_manager = start_job_manager(&agent);
    let channel = Arc::new(MockChannel::with_acl(true, std::iter::empty::<&str>()));
    let channel_dyn: Arc<dyn Channel> = channel.clone();

    process_discord_message(
        agent.clone(),
        channel_dyn.clone(),
        inbound("/session admin set 1001,1002"),
        &job_manager,
        10,
    )
    .await;
    process_discord_message(
        agent,
        channel_dyn,
        inbound("/session admin json"),
        &job_manager,
        10,
    )
    .await;

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 2);
    assert!(sent[0].0.contains("Session delegated admins updated."));
    let payload: serde_json::Value = serde_json::from_str(&sent[1].0)?;
    assert_eq!(payload["kind"], "session_admin");
    assert_eq!(payload["updated"], false);
    assert_eq!(
        payload["override_admin_users"],
        serde_json::json!(["1001", "1002"])
    );
    Ok(())
}

#[tokio::test]
async fn process_discord_message_handles_session_injection_set_and_status_json() -> Result<()> {
    let agent = build_agent().await?;
    let job_manager = start_job_manager(&agent);
    let channel = Arc::new(MockChannel::with_acl(true, std::iter::empty::<&str>()));
    let channel_dyn: Arc<dyn Channel> = channel.clone();

    process_discord_message(
        agent.clone(),
        channel_dyn.clone(),
        inbound("/session inject <qa><q>backend</q><a>valkey</a></qa>"),
        &job_manager,
        10,
    )
    .await;
    process_discord_message(
        agent,
        channel_dyn,
        inbound("/session inject status json"),
        &job_manager,
        10,
    )
    .await;

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 2);
    assert!(
        sent[0]
            .0
            .contains("Session system prompt injection updated.")
    );
    let payload: serde_json::Value = serde_json::from_str(&sent[1].0)?;
    assert_eq!(payload["kind"], "session_injection");
    assert_eq!(payload["configured"], true);
    assert_eq!(payload["qa_count"], 1);
    Ok(())
}
