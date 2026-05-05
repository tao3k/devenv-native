//! Telegram webhook branch for app, builders, handler, and serving.

mod app;
mod auth;
mod builders;
mod dedup;
mod handler;
mod path;
mod state;

pub use app::TelegramWebhookApp;
pub use builders::{
    TelegramWebhookControlPolicyBuildRequest, TelegramWebhookPartitionBuildRequest,
    build_telegram_webhook_app, build_telegram_webhook_app_with_control_command_policy,
    build_telegram_webhook_app_with_partition,
};
