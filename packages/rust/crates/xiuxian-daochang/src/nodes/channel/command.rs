use std::path::PathBuf;

use crate::{
    ChannelProvider, DiscordRuntimeMode, RuntimeSettings, TelegramChannelMode,
    WebhookDedupBackendMode,
};

use super::{discord, telegram};

pub(crate) struct ChannelCommandRequest {
    pub(crate) provider: ChannelProvider,
    pub(crate) bot_token: Option<String>,
    pub(crate) tool_config: PathBuf,
    pub(crate) mode: Option<TelegramChannelMode>,
    pub(crate) webhook_bind: Option<String>,
    pub(crate) webhook_path: Option<String>,
    pub(crate) webhook_secret_token: Option<String>,
    pub(crate) session_partition: Option<String>,
    pub(crate) inbound_queue_capacity: Option<usize>,
    pub(crate) turn_timeout_secs: Option<u64>,
    pub(crate) discord_runtime_mode: Option<DiscordRuntimeMode>,
    pub(crate) webhook_dedup_backend: Option<WebhookDedupBackendMode>,
    pub(crate) valkey_url: Option<String>,
    pub(crate) webhook_dedup_ttl_secs: Option<u64>,
    pub(crate) webhook_dedup_key_prefix: Option<String>,
}

pub(crate) async fn run_channel_command(
    req: ChannelCommandRequest,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<()> {
    let command_future: std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>,
    > = match req.provider {
        ChannelProvider::Telegram => Box::pin(telegram::run_telegram_channel_command(
            req,
            runtime_settings,
        )),
        ChannelProvider::Discord => {
            Box::pin(discord::run_discord_channel_command(req, runtime_settings))
        }
    };
    command_future.await
}
