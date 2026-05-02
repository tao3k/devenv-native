//! Discord ingress loop wiring from Serenity events into managed runtime turns.

use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use anyhow::{Result, ensure};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use tokio::sync::mpsc;
use tokio::time::timeout;

use crate::channels::traits::ChannelMessage;

use crate::channels::discord::channel::{DiscordChannel, DiscordControlCommandPolicy};
use crate::channels::discord::session_partition::DiscordSessionPartition;

const DISCORD_INGRESS_SECRET_HEADER: &str = "x-xiuxian-discord-ingress-token";
const DISCORD_INGRESS_INBOUND_SEND_TIMEOUT_SECS: u64 = 2;

/// Built ingress components for Discord handler testing and runtime wiring.
pub struct DiscordIngressApp {
    /// Axum router that serves Discord ingress endpoint.
    pub app: Router,
    /// Discord channel instance used by this ingress app.
    pub channel: Arc<DiscordChannel>,
    /// Normalized ingress route path.
    pub path: String,
}

/// Full configuration for building Discord ingress with explicit partition and policy.
pub struct DiscordIngressBuildRequest {
    /// Bot token used by outbound Discord API calls.
    pub bot_token: String,
    /// Optional allowlist of user ids.
    pub allowed_users: Vec<String>,
    /// Optional allowlist of guild ids.
    pub allowed_guilds: Vec<String>,
    /// Policy for control and slash managed commands.
    pub control_command_policy: DiscordControlCommandPolicy,
    /// Ingress route path.
    pub ingress_path: String,
    /// Optional shared secret token for ingress validation.
    pub secret_token: Option<String>,
    /// Session partition strategy.
    pub session_partition: DiscordSessionPartition,
    /// Inbound queue sender for parsed channel messages.
    pub tx: mpsc::Sender<ChannelMessage>,
}

/// Build a Discord ingress app.
///
/// # Errors
/// Returns an error when channel/runtime initialization fails.
pub fn build_discord_ingress_app(
    bot_token: String,
    allowed_users: Vec<String>,
    allowed_guilds: Vec<String>,
    ingress_path: &str,
    secret_token: Option<String>,
    tx: mpsc::Sender<ChannelMessage>,
) -> Result<DiscordIngressApp> {
    let admin_users = allowed_users.clone();
    build_discord_ingress_app_with_control_command_policy(
        bot_token,
        allowed_users,
        allowed_guilds,
        DiscordControlCommandPolicy::new(admin_users, None, Vec::new()),
        ingress_path,
        secret_token,
        tx,
    )
}

/// Build a Discord ingress app with explicit control-command policy.
///
/// # Errors
/// Returns an error when channel/runtime initialization fails.
pub fn build_discord_ingress_app_with_control_command_policy(
    bot_token: String,
    allowed_users: Vec<String>,
    allowed_guilds: Vec<String>,
    control_command_policy: DiscordControlCommandPolicy,
    ingress_path: &str,
    secret_token: Option<String>,
    tx: mpsc::Sender<ChannelMessage>,
) -> Result<DiscordIngressApp> {
    build_discord_ingress_app_with_partition_and_control_command_policy(
        DiscordIngressBuildRequest {
            bot_token,
            allowed_users,
            allowed_guilds,
            control_command_policy,
            ingress_path: ingress_path.to_string(),
            secret_token,
            session_partition: DiscordSessionPartition::from_env(),
            tx,
        },
    )
}

/// Build a Discord ingress app with explicit session partition and control-command policy.
#[doc(hidden)]
///
/// # Errors
/// Returns an error when channel/runtime initialization fails.
pub fn build_discord_ingress_app_with_partition_and_control_command_policy(
    request: DiscordIngressBuildRequest,
) -> Result<DiscordIngressApp> {
    let DiscordIngressBuildRequest {
        bot_token,
        allowed_users,
        allowed_guilds,
        control_command_policy,
        ingress_path,
        secret_token,
        session_partition,
        tx,
    } = request;

    ensure!(
        !bot_token.trim().is_empty(),
        "discord bot token cannot be empty"
    );

    let channel = Arc::new(
        DiscordChannel::new_with_partition_and_control_command_policy(
            bot_token,
            allowed_users,
            allowed_guilds,
            control_command_policy,
            session_partition,
        ),
    );
    let ingress_state = DiscordIngressState {
        channel: Arc::clone(&channel),
        tx,
        secret_token,
    };

    let path = normalize_ingress_path(&ingress_path);
    let app = Router::new()
        .route(&path, post(discord_ingress_handler))
        .with_state(ingress_state);

    Ok(DiscordIngressApp { app, channel, path })
}

#[derive(Clone)]
struct DiscordIngressState {
    channel: Arc<DiscordChannel>,
    tx: mpsc::Sender<ChannelMessage>,
    secret_token: Option<String>,
}

fn normalize_ingress_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        "/discord/ingress".to_string()
    } else if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    }
}

async fn discord_ingress_handler(
    State(state): State<DiscordIngressState>,
    headers: HeaderMap,
    Json(event): Json<serde_json::Value>,
) -> Result<StatusCode, (StatusCode, String)> {
    validate_ingress_secret(&headers, state.secret_token.as_deref())?;

    if let Some(msg) = state.channel.parse_gateway_message(&event) {
        enqueue_ingress_message(&state.tx, msg).await?;
    } else {
        log_ignored_ingress_event(&event);
    }

    Ok(StatusCode::OK)
}

fn validate_ingress_secret(
    headers: &HeaderMap,
    expected_secret: Option<&str>,
) -> Result<(), (StatusCode, String)> {
    let Some(expected) = expected_secret else {
        return Ok(());
    };
    let provided = headers
        .get(DISCORD_INGRESS_SECRET_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    if provided == expected {
        return Ok(());
    }

    Err((
        StatusCode::UNAUTHORIZED,
        "invalid discord ingress secret token".to_string(),
    ))
}

async fn enqueue_ingress_message(
    tx: &mpsc::Sender<ChannelMessage>,
    msg: ChannelMessage,
) -> Result<(), (StatusCode, String)> {
    let session_key = msg.session_key.clone();
    let recipient = msg.recipient.clone();
    tracing::info!(
        channel = "discord",
        session_key = %session_key,
        recipient = %recipient,
        "discord ingress parsed message"
    );

    let send_started = Instant::now();
    match timeout(
        Duration::from_secs(DISCORD_INGRESS_INBOUND_SEND_TIMEOUT_SECS),
        tx.send(msg),
    )
    .await
    {
        Ok(Ok(())) => {}
        Ok(Err(_)) => {
            tracing::error!("discord inbound queue unavailable");
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                "discord inbound queue unavailable".to_string(),
            ));
        }
        Err(_) => {
            tracing::warn!(
                event = "discord.ingress.inbound_queue_timeout",
                timeout_secs = DISCORD_INGRESS_INBOUND_SEND_TIMEOUT_SECS,
                session_key = %session_key,
                recipient = %recipient,
                "discord ingress inbound queue send timed out"
            );
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                "discord inbound queue send timed out".to_string(),
            ));
        }
    }

    let send_wait_ms = u64::try_from(send_started.elapsed().as_millis()).unwrap_or(u64::MAX);
    if send_wait_ms >= 50 {
        tracing::warn!(
            event = "discord.ingress.inbound_queue_wait",
            wait_ms = send_wait_ms,
            session_key = %session_key,
            recipient = %recipient,
            "discord ingress waited on inbound queue send"
        );
    }
    Ok(())
}

fn log_ignored_ingress_event(event: &serde_json::Value) {
    let event_type = event
        .get("type")
        .and_then(serde_json::Value::as_u64)
        .map_or_else(|| "(none)".to_string(), |value| value.to_string());
    let message_id = event
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("(none)");
    let channel_id = event
        .get("channel_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("(none)");
    let guild_id = event
        .get("guild_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("(none)");
    let content_len = event
        .get("content")
        .and_then(serde_json::Value::as_str)
        .map_or(0, str::len);
    let attachment_count = event
        .get("attachments")
        .and_then(serde_json::Value::as_array)
        .map_or(0, Vec::len);
    tracing::warn!(
        event = "discord.ingress.event.ignored",
        event_type = %event_type,
        message_id = %message_id,
        channel_id = %channel_id,
        guild_id = %guild_id,
        content_len,
        attachments = attachment_count,
        "discord ingress ignored event during parsing"
    );
}
