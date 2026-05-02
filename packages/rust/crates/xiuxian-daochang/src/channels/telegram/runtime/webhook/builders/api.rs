//! Telegram webhook API client builder.

use anyhow::Result;
use tokio::sync::mpsc;

use crate::channels::telegram::idempotency::WebhookDedupConfig;
use crate::channels::telegram::session_partition::TelegramSessionPartition;
use crate::channels::telegram::{TelegramCommandAdminRule, TelegramControlCommandPolicy};
use crate::channels::traits::ChannelMessage;

use super::core::{
    TelegramWebhookCoreBuildRequest,
    build_telegram_webhook_app_with_partition_and_control_command_policy,
};
use crate::channels::telegram::runtime::webhook::app::TelegramWebhookApp;

/// Request payload for building a Telegram webhook app with an explicit control
/// command policy.
pub struct TelegramWebhookControlPolicyBuildRequest {
    /// Telegram bot token used for webhook API calls.
    pub bot_token: String,
    /// Allow-listed Telegram users.
    pub allowed_users: Vec<String>,
    /// Allow-listed Telegram groups or chats.
    pub allowed_groups: Vec<String>,
    /// Prebuilt control-command authorization policy.
    pub control_command_policy: TelegramControlCommandPolicy,
    /// Webhook path mounted on the HTTP server.
    pub webhook_path: String,
    /// Optional Telegram secret token used to validate webhook requests.
    pub secret_token: Option<String>,
    /// Deduplication backend configuration for inbound webhook updates.
    pub dedup_config: WebhookDedupConfig,
    /// Channel message sink that receives accepted inbound Telegram updates.
    pub tx: mpsc::Sender<ChannelMessage>,
}

/// Request payload for building a Telegram webhook app with an explicit session
/// partition strategy.
pub struct TelegramWebhookPartitionBuildRequest {
    /// Telegram bot token used for webhook API calls.
    pub bot_token: String,
    /// Allow-listed Telegram users.
    pub allowed_users: Vec<String>,
    /// Allow-listed Telegram groups or chats.
    pub allowed_groups: Vec<String>,
    /// Admin users allowed to execute privileged control commands.
    pub admin_users: Vec<String>,
    /// Webhook path mounted on the HTTP server.
    pub webhook_path: String,
    /// Optional Telegram secret token used to validate webhook requests.
    pub secret_token: Option<String>,
    /// Deduplication backend configuration for inbound webhook updates.
    pub dedup_config: WebhookDedupConfig,
    /// Session partition mode used to derive logical conversation keys.
    pub session_partition: TelegramSessionPartition,
    /// Channel message sink that receives accepted inbound Telegram updates.
    pub tx: mpsc::Sender<ChannelMessage>,
}

/// Builds a Telegram webhook application with the default session partition and
/// an empty control-command admin policy.
///
/// # Errors
///
/// Returns an error when webhook core construction fails.
pub fn build_telegram_webhook_app(
    bot_token: String,
    allowed_users: Vec<String>,
    allowed_groups: Vec<String>,
    webhook_path: impl Into<String>,
    secret_token: Option<String>,
    dedup_config: WebhookDedupConfig,
    tx: mpsc::Sender<ChannelMessage>,
) -> Result<TelegramWebhookApp> {
    build_telegram_webhook_app_with_control_command_policy(
        TelegramWebhookControlPolicyBuildRequest {
            bot_token,
            allowed_users,
            allowed_groups,
            control_command_policy: TelegramControlCommandPolicy::new(
                Vec::new(),
                None,
                Vec::<TelegramCommandAdminRule>::new(),
            ),
            webhook_path: webhook_path.into(),
            secret_token,
            dedup_config,
            tx,
        },
    )
}

/// Builds a Telegram webhook application with an explicit control-command
/// policy.
///
/// # Errors
///
/// Returns an error when webhook core construction fails.
pub fn build_telegram_webhook_app_with_control_command_policy(
    request: TelegramWebhookControlPolicyBuildRequest,
) -> Result<TelegramWebhookApp> {
    let TelegramWebhookControlPolicyBuildRequest {
        bot_token,
        allowed_users,
        allowed_groups,
        control_command_policy,
        webhook_path,
        secret_token,
        dedup_config,
        tx,
    } = request;
    build_telegram_webhook_app_with_partition_and_control_command_policy(
        TelegramWebhookCoreBuildRequest {
            bot_token,
            allowed_users,
            allowed_groups,
            control_command_policy,
            webhook_path,
            secret_token,
            dedup_config,
            session_partition: TelegramSessionPartition::default(),
            tx,
        },
    )
}

/// Builds a Telegram webhook application with an explicit session partition and
/// admin-user list.
///
/// # Errors
///
/// Returns an error when webhook core construction fails.
pub fn build_telegram_webhook_app_with_partition(
    request: TelegramWebhookPartitionBuildRequest,
) -> Result<TelegramWebhookApp> {
    let TelegramWebhookPartitionBuildRequest {
        bot_token,
        allowed_users,
        allowed_groups,
        admin_users,
        webhook_path,
        secret_token,
        dedup_config,
        session_partition,
        tx,
    } = request;
    build_telegram_webhook_app_with_partition_and_control_command_policy(
        TelegramWebhookCoreBuildRequest {
            bot_token,
            allowed_users,
            allowed_groups,
            control_command_policy: TelegramControlCommandPolicy::new(
                admin_users,
                None,
                Vec::<TelegramCommandAdminRule>::new(),
            ),
            webhook_path,
            secret_token,
            dedup_config,
            session_partition,
            tx,
        },
    )
}
