//! Test coverage for xiuxian-daochang behavior.

use anyhow::{Result, bail};

use xiuxian_daochang::{
    TelegramControlCommandPolicy, TelegramWebhookPolicyRunRequest, WebhookDedupBackend,
    WebhookDedupConfig, run_telegram_webhook_with_control_command_policy,
};

use super::build_agent;

#[tokio::test]
async fn runtime_webhook_requires_non_empty_secret_token() -> Result<()> {
    let agent = build_agent().await?;
    let error =
        match run_telegram_webhook_with_control_command_policy(TelegramWebhookPolicyRunRequest {
            agent,
            bot_token: "fake-token".to_string(),
            allowed_users: vec!["*".to_string()],
            allowed_groups: vec![],
            control_command_policy: TelegramControlCommandPolicy::default(),
            bind_addr: "127.0.0.1:0".to_string(),
            webhook_path: "/telegram/webhook".to_string(),
            secret_token: None,
            dedup_config: WebhookDedupConfig {
                backend: WebhookDedupBackend::Memory,
                ttl_secs: 600,
            },
        })
        .await
        {
            Ok(()) => bail!("missing webhook secret should fail before starting runtime"),
            Err(error) => error,
        };

    assert!(
        error
            .to_string()
            .contains("requires a non-empty secret token"),
        "unexpected error: {error}"
    );
    Ok(())
}
