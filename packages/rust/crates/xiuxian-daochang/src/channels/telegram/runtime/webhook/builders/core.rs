use std::sync::Arc;

use anyhow::Result;
use axum::{Extension, Router, routing::post};
use tokio::sync::mpsc;

use crate::channels::telegram::idempotency::WebhookDedupConfig;
use crate::channels::telegram::session_partition::TelegramSessionPartition;
use crate::channels::telegram::{TelegramChannel, TelegramControlCommandPolicy};
use crate::channels::traits::ChannelMessage;
use crate::gateway::{embedding_routes, new_embedding_runtime};

use crate::channels::telegram::runtime::webhook::app::TelegramWebhookApp;
use crate::channels::telegram::runtime::webhook::handler::telegram_webhook_handler;
use crate::channels::telegram::runtime::webhook::path::normalize_webhook_path;
use crate::channels::telegram::runtime::webhook::state::TelegramWebhookState;

pub(super) struct TelegramWebhookCoreBuildRequest {
    pub(super) bot_token: String,
    pub(super) allowed_users: Vec<String>,
    pub(super) allowed_groups: Vec<String>,
    pub(super) control_command_policy: TelegramControlCommandPolicy,
    pub(super) webhook_path: String,
    pub(super) secret_token: Option<String>,
    pub(super) dedup_config: WebhookDedupConfig,
    pub(super) session_partition: TelegramSessionPartition,
    pub(super) tx: mpsc::Sender<ChannelMessage>,
}

pub(super) fn build_telegram_webhook_app_with_partition_and_control_command_policy(
    request: TelegramWebhookCoreBuildRequest,
) -> Result<TelegramWebhookApp> {
    let TelegramWebhookCoreBuildRequest {
        bot_token,
        allowed_users,
        allowed_groups,
        control_command_policy,
        webhook_path,
        secret_token,
        dedup_config,
        session_partition,
        tx,
    } = request;

    let dedup_config = dedup_config.normalized();
    let deduplicator = dedup_config.build_store()?;
    let channel = Arc::new(
        TelegramChannel::new_with_partition_and_control_command_policy(
            bot_token,
            allowed_users,
            allowed_groups,
            control_command_policy,
            session_partition,
        ),
    );
    let webhook_state = TelegramWebhookState {
        channel: Arc::clone(&channel),
        tx,
        secret_token,
        deduplicator,
    };

    let path = normalize_webhook_path(&webhook_path);
    let embedding_runtime = new_embedding_runtime();
    let app = Router::new()
        .route(&path, post(telegram_webhook_handler))
        .merge(embedding_routes::<TelegramWebhookState>())
        .layer(Extension(embedding_runtime))
        .with_state(webhook_state);

    Ok(TelegramWebhookApp {
        app,
        channel,
        path,
        dedup_config,
    })
}
