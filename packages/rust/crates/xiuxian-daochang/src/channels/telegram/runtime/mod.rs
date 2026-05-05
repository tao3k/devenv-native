//! Telegram runtime coordinates polling, webhook ingress, foreground delivery, and session policy.

mod console;
mod dispatch;
pub(crate) mod jobs;
mod run_polling;
mod run_webhook;
mod telemetry;
mod test_api;
mod webhook;

pub(crate) use dispatch::ForegroundInterruptController;
pub use run_polling::{run_telegram, run_telegram_with_control_command_policy};
pub use run_webhook::{
    TelegramWebhookPolicyRunRequest, TelegramWebhookRunRequest, run_telegram_webhook,
    run_telegram_webhook_with_control_command_policy,
};
pub(crate) use test_api::{
    test_handle_inbound_message_with_interrupt, test_log_preview, test_push_background_completion,
    test_resolve_snapshot_interval_secs,
};
pub use webhook::{
    TelegramWebhookApp, TelegramWebhookControlPolicyBuildRequest,
    TelegramWebhookPartitionBuildRequest, build_telegram_webhook_app,
    build_telegram_webhook_app_with_control_command_policy,
    build_telegram_webhook_app_with_partition,
};
