//! Telegram webhook builder branch for API, channel, and core assembly.

mod api;
mod core;

pub use api::{
    TelegramWebhookControlPolicyBuildRequest, TelegramWebhookPartitionBuildRequest,
    build_telegram_webhook_app, build_telegram_webhook_app_with_control_command_policy,
    build_telegram_webhook_app_with_partition,
};
