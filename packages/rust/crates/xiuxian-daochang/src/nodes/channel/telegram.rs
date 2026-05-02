use std::path::PathBuf;
use std::sync::Arc;

use crate::{
    DEFAULT_REDIS_KEY_PREFIX, RuntimeSettings, TelegramControlCommandPolicy,
    TelegramSlashCommandPolicy, TelegramWebhookPolicyRunRequest, WebhookDedupBackend,
    WebhookDedupConfig, build_telegram_acl_overrides,
    run_telegram_webhook_with_control_command_policy, run_telegram_with_control_command_policy,
};
use xiuxian_macros::env_non_empty;

use crate::build_agent;
use crate::resolve::{
    resolve_channel_mode, resolve_dedup_backend, resolve_positive_u64, resolve_string,
    resolve_valkey_url_env,
};
use crate::{TelegramChannelMode, WebhookDedupBackendMode};

use super::ChannelCommandRequest;
use super::runtime_guard::{
    apply_channel_embedding_memory_guard, log_control_command_allow_override,
    log_slash_command_allow_override,
};

struct TelegramChannelRunRequest {
    bot_token: String,
    allowed_users: Vec<String>,
    allowed_groups: Vec<String>,
    control_command_policy: TelegramControlCommandPolicy,
    tool_config_path: PathBuf,
    mode: TelegramChannelMode,
    webhook_bind: String,
    webhook_path: String,
    webhook_secret_token: Option<String>,
    webhook_dedup_config: WebhookDedupConfig,
}

pub(super) async fn run_telegram_channel_command(
    req: ChannelCommandRequest,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<()> {
    let ChannelCommandRequest {
        bot_token,
        tool_config,
        mode,
        webhook_bind,
        webhook_path,
        webhook_secret_token,
        webhook_dedup_backend,
        valkey_url,
        webhook_dedup_ttl_secs,
        webhook_dedup_key_prefix,
        ..
    } = req;

    let acl_overrides = build_telegram_acl_overrides(runtime_settings)?;
    let channel_mode = resolve_channel_mode(mode, runtime_settings.telegram.mode.as_deref());
    let webhook_bind_addr = resolve_string(
        webhook_bind,
        "XIUXIAN_DAOCHANG_TELEGRAM_WEBHOOK_BIND",
        runtime_settings.telegram.webhook_bind.as_deref(),
        "localhost:8081",
    );
    let webhook_route_path = resolve_string(
        webhook_path,
        "XIUXIAN_DAOCHANG_TELEGRAM_WEBHOOK_PATH",
        runtime_settings.telegram.webhook_path.as_deref(),
        "/telegram/webhook",
    );
    let dedup_backend_mode = resolve_dedup_backend(
        webhook_dedup_backend,
        runtime_settings.telegram.webhook_dedup_backend.as_deref(),
    );
    let dedup_ttl_secs = resolve_positive_u64(
        webhook_dedup_ttl_secs,
        "XIUXIAN_DAOCHANG_TELEGRAM_WEBHOOK_DEDUP_TTL_SECS",
        runtime_settings.telegram.webhook_dedup_ttl_secs,
        600,
    );
    let dedup_key_prefix = resolve_string(
        webhook_dedup_key_prefix,
        "XIUXIAN_DAOCHANG_TELEGRAM_WEBHOOK_DEDUP_KEY_PREFIX",
        runtime_settings
            .telegram
            .webhook_dedup_key_prefix
            .as_deref(),
        DEFAULT_REDIS_KEY_PREFIX,
    );
    let resolved_bot_token = bot_token
        .or_else(|| env_non_empty!("TELEGRAM_BOT_TOKEN"))
        .ok_or_else(|| anyhow::anyhow!("--bot-token or TELEGRAM_BOT_TOKEN required"))?;
    let resolved_webhook_secret = resolve_webhook_secret_token(channel_mode, webhook_secret_token)?;
    let dedup_config = build_webhook_dedup_config(
        dedup_backend_mode,
        valkey_url,
        dedup_ttl_secs,
        dedup_key_prefix,
        runtime_settings,
    )?;

    let control_command_allow_entries = acl_overrides.control_command_allow_from;
    log_control_command_allow_override("telegram", control_command_allow_entries.as_deref());
    let slash_global_allow_entries = acl_overrides.slash_command_allow_from;
    log_slash_command_allow_override("telegram", slash_global_allow_entries.as_deref());
    let slash_command_policy = TelegramSlashCommandPolicy {
        global: slash_global_allow_entries,
        session_status: acl_overrides.slash_session_status_allow_from,
        session_budget: acl_overrides.slash_session_budget_allow_from,
        session_memory: acl_overrides.slash_session_memory_allow_from,
        session_feedback: acl_overrides.slash_session_feedback_allow_from,
        job_status: acl_overrides.slash_job_allow_from,
        jobs_summary: acl_overrides.slash_jobs_allow_from,
        background_submit: acl_overrides.slash_bg_allow_from,
    };
    let control_command_policy = TelegramControlCommandPolicy::new(
        acl_overrides.admin_users,
        control_command_allow_entries,
        acl_overrides.control_command_rules,
    )
    .with_slash_command_policy(slash_command_policy);

    Box::pin(run_telegram_channel_mode(
        TelegramChannelRunRequest {
            bot_token: resolved_bot_token,
            allowed_users: acl_overrides.allowed_users,
            allowed_groups: acl_overrides.allowed_groups,
            control_command_policy,
            tool_config_path: tool_config,
            mode: channel_mode,
            webhook_bind: webhook_bind_addr,
            webhook_path: webhook_route_path,
            webhook_secret_token: resolved_webhook_secret,
            webhook_dedup_config: dedup_config,
        },
        runtime_settings,
    ))
    .await
}

fn resolve_webhook_secret_token(
    channel_mode: TelegramChannelMode,
    cli_secret: Option<String>,
) -> anyhow::Result<Option<String>> {
    let secret = cli_secret.or_else(|| env_non_empty!("TELEGRAM_WEBHOOK_SECRET"));
    if matches!(channel_mode, TelegramChannelMode::Webhook) && secret.is_none() {
        return Err(anyhow::anyhow!(
            "webhook mode requires TELEGRAM_WEBHOOK_SECRET (or --webhook-secret-token)"
        ));
    }
    Ok(secret)
}

async fn run_telegram_channel_mode(
    request: TelegramChannelRunRequest,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<()> {
    let TelegramChannelRunRequest {
        bot_token,
        allowed_users,
        allowed_groups,
        control_command_policy,
        tool_config_path,
        mode,
        webhook_bind,
        webhook_path,
        webhook_secret_token,
        webhook_dedup_config,
    } = request;

    let effective_runtime_settings = apply_channel_embedding_memory_guard(runtime_settings);
    let agent = Arc::new(build_agent(&tool_config_path, &effective_runtime_settings).await?);
    if allowed_users.is_empty() && allowed_groups.is_empty() {
        tracing::warn!(
            "Telegram ACL allowlist is empty; all inbound will be rejected. \
             Configure `telegram.acl.allow.users` or `telegram.acl.allow.groups` to allow traffic."
        );
    }
    match mode {
        TelegramChannelMode::Polling => {
            run_telegram_with_control_command_policy(
                Arc::clone(&agent),
                bot_token,
                allowed_users,
                allowed_groups,
                control_command_policy,
            )
            .await
        }
        TelegramChannelMode::Webhook => {
            run_telegram_webhook_with_control_command_policy(TelegramWebhookPolicyRunRequest {
                agent: Arc::clone(&agent),
                bot_token,
                allowed_users,
                allowed_groups,
                control_command_policy,
                bind_addr: webhook_bind,
                webhook_path,
                secret_token: webhook_secret_token,
                dedup_config: webhook_dedup_config,
            })
            .await
        }
    }
}

fn build_webhook_dedup_config(
    backend_mode: WebhookDedupBackendMode,
    valkey_url: Option<String>,
    ttl_secs: u64,
    key_prefix: String,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<WebhookDedupConfig> {
    let backend = match backend_mode {
        WebhookDedupBackendMode::Memory => WebhookDedupBackend::Memory,
        WebhookDedupBackendMode::Valkey => {
            let url = valkey_url
                .or_else(|| runtime_settings.session.valkey_url.clone())
                .or_else(resolve_valkey_url_env)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "valkey dedup backend requires valkey url (explicit --valkey-url, session.valkey_url, XIUXIAN_WENDAO_VALKEY_URL, or VALKEY_URL)"
                    )
                })?;
            if url.trim().is_empty() {
                return Err(anyhow::anyhow!(
                    "valkey dedup backend requires a non-empty URL"
                ));
            }
            WebhookDedupBackend::Redis { url, key_prefix }
        }
    };

    Ok(WebhookDedupConfig { backend, ttl_secs })
}
