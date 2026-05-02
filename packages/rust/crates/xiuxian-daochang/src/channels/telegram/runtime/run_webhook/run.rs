//! Telegram webhook runtime startup loop.

use std::sync::Arc;

use anyhow::Result;

use crate::agent::Agent;
use crate::channels::telegram::TelegramCommandAdminRule;
use crate::channels::telegram::TelegramControlCommandPolicy;
use crate::channels::telegram::idempotency::WebhookDedupConfig;
use crate::channels::telegram::runtime::console::{
    print_foreground_config, print_managed_commands_help,
};
use crate::channels::telegram::runtime::dispatch::start_telegram_runtime;
use crate::channels::telegram::runtime::{
    TelegramWebhookControlPolicyBuildRequest,
    build_telegram_webhook_app_with_control_command_policy,
};
use crate::channels::telegram::runtime_config::TelegramRuntimeConfig;
use crate::channels::traits::Channel;

use super::loop_control;
use super::secret;
use super::server;

/// Full webhook run request with legacy admin-user and command-rule inputs.
pub struct TelegramWebhookRunRequest {
    /// Shared agent runtime.
    pub agent: Arc<Agent>,
    /// Telegram bot token.
    pub bot_token: String,
    /// Allowed sender user identifiers.
    pub allowed_users: Vec<String>,
    /// Allowed group identifiers.
    pub allowed_groups: Vec<String>,
    /// Fallback admin users for control/slash ACL.
    pub admin_users: Vec<String>,
    /// Optional global control-command allow identities.
    pub control_command_allow_from: Option<Vec<String>>,
    /// Command-scoped control rules.
    pub control_command_rules: Vec<TelegramCommandAdminRule>,
    /// Webhook bind address.
    pub bind_addr: String,
    /// Webhook route path.
    pub webhook_path: String,
    /// Optional webhook secret token.
    pub secret_token: Option<String>,
    /// Webhook dedup backend configuration.
    pub dedup_config: WebhookDedupConfig,
}

/// Webhook run request with explicit structured control-command policy.
pub struct TelegramWebhookPolicyRunRequest {
    /// Shared agent runtime.
    pub agent: Arc<Agent>,
    /// Telegram bot token.
    pub bot_token: String,
    /// Allowed sender user identifiers.
    pub allowed_users: Vec<String>,
    /// Allowed group identifiers.
    pub allowed_groups: Vec<String>,
    /// Structured control/slash command policy.
    pub control_command_policy: TelegramControlCommandPolicy,
    /// Webhook bind address.
    pub bind_addr: String,
    /// Webhook route path.
    pub webhook_path: String,
    /// Optional webhook secret token.
    pub secret_token: Option<String>,
    /// Webhook dedup backend configuration.
    pub dedup_config: WebhookDedupConfig,
}

/// Run Telegram channel via webhook (recommended for multi-instance deployments).
///
/// # Errors
/// Returns an error when runtime initialization or webhook server startup fails.
pub async fn run_telegram_webhook(request: TelegramWebhookRunRequest) -> Result<()> {
    let TelegramWebhookRunRequest {
        agent,
        bot_token,
        allowed_users,
        allowed_groups,
        admin_users,
        control_command_allow_from,
        control_command_rules,
        bind_addr,
        webhook_path,
        secret_token,
        dedup_config,
    } = request;
    run_telegram_webhook_with_control_command_policy(TelegramWebhookPolicyRunRequest {
        agent,
        bot_token,
        allowed_users,
        allowed_groups,
        control_command_policy: TelegramControlCommandPolicy::new(
            admin_users,
            control_command_allow_from,
            control_command_rules,
        ),
        bind_addr,
        webhook_path,
        secret_token,
        dedup_config,
    })
    .await
}

/// Run Telegram channel via webhook with structured control-command policy.
///
/// # Errors
/// Returns an error when runtime initialization or webhook server startup fails.
pub async fn run_telegram_webhook_with_control_command_policy(
    request: TelegramWebhookPolicyRunRequest,
) -> Result<()> {
    let TelegramWebhookPolicyRunRequest {
        agent,
        bot_token,
        allowed_users,
        allowed_groups,
        control_command_policy,
        bind_addr,
        webhook_path,
        secret_token,
        dedup_config,
    } = request;
    let secret_token = secret::normalize_secret_token(secret_token)?;
    let runtime_config = TelegramRuntimeConfig::from_env();
    let (tx, mut inbound_rx) = tokio::sync::mpsc::channel(runtime_config.inbound_queue_capacity);
    let webhook = build_telegram_webhook_app_with_control_command_policy(
        TelegramWebhookControlPolicyBuildRequest {
            bot_token,
            allowed_users,
            allowed_groups,
            control_command_policy,
            webhook_path,
            secret_token: Some(secret_token),
            dedup_config,
            tx,
        },
    )?;
    let channel_for_send: Arc<dyn Channel> = webhook.channel.clone();
    let session_partition = webhook.channel.session_partition();
    let path = webhook.path;
    let dedup_config = webhook.dedup_config;
    let app = webhook.app;

    let mut webhook_server = server::start_webhook_server(&bind_addr, app).await?;

    let (
        session_gate_backend,
        foreground_tx,
        interrupt_controller,
        foreground_dispatcher,
        job_manager,
        mut completion_rx,
    ) = start_telegram_runtime(Arc::clone(&agent), &channel_for_send, runtime_config)?;

    println!("Telegram webhook listening on {bind_addr}{path} (Ctrl+C to stop)");
    let backend_name = dedup_config.backend_name();
    let ttl_secs = dedup_config.ttl_secs;
    println!("Webhook dedup backend: {backend_name} (ttl={ttl_secs}s)");
    println!("Session partition: {session_partition}");
    print_foreground_config(&runtime_config, &session_gate_backend);
    print_managed_commands_help();

    loop_control::run_webhook_event_loop(
        loop_control::WebhookLoopReceivers {
            inbound_rx: &mut inbound_rx,
            completion_rx: &mut completion_rx,
        },
        loop_control::WebhookLoopContext {
            channel_for_send: &channel_for_send,
            foreground_tx: &foreground_tx,
            interrupt_controller: &interrupt_controller,
            job_manager: &job_manager,
            agent: &agent,
            foreground_queue_mode: runtime_config.foreground_queue_mode,
            webhook_server: &mut webhook_server.task,
        },
    )
    .await;

    server::stop_webhook_server(webhook_server).await;
    drop(foreground_tx);
    foreground_dispatcher.abort();
    Ok(())
}
