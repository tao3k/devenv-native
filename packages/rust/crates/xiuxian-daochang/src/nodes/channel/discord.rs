use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::{
    DiscordCommandAdminRule, DiscordControlCommandPolicy, DiscordIngressRunRequest,
    DiscordRuntimeConfig, DiscordSessionPartition, DiscordSlashCommandPolicy, ForegroundQueueMode,
    RuntimeSettings, build_discord_acl_overrides, run_discord_gateway, run_discord_ingress,
};
use xiuxian_macros::env_non_empty;

use crate::DiscordRuntimeMode;
use crate::build_agent;
use crate::resolve::{
    resolve_bool, resolve_discord_runtime_mode, resolve_positive_u64, resolve_positive_usize,
    resolve_string,
};

use super::ChannelCommandRequest;
use super::runtime_guard::{
    apply_channel_embedding_memory_guard, log_control_command_allow_override,
    log_slash_command_allow_override,
};

const DISCORD_DEFAULT_INBOUND_QUEUE_CAPACITY: usize = 512;
const DISCORD_DEFAULT_TURN_TIMEOUT_SECS: u64 = 120;
const DISCORD_DEFAULT_FOREGROUND_MAX_IN_FLIGHT_MESSAGES: usize = 16;
const DISCORD_DEFAULT_INGRESS_BIND: &str = "0.0.0.0:18082";
const DISCORD_DEFAULT_INGRESS_PATH: &str = "/discord/ingress";

struct ResolvedDiscordRuntimeConfig {
    runtime_config: DiscordRuntimeConfig,
    runtime_mode: DiscordRuntimeMode,
}

struct DiscordRuntimeLaunchConfig {
    bot_token: String,
    tool_config_path: PathBuf,
    runtime_mode: DiscordRuntimeMode,
    runtime_config: DiscordRuntimeConfig,
    ingress_bind: String,
    ingress_path: String,
    ingress_secret_token: Option<String>,
}

struct DiscordAclLaunchConfig {
    allowed_users: Vec<String>,
    allowed_guilds: Vec<String>,
    admin_users: Vec<String>,
    control_command_allow_from: Option<Vec<String>>,
    control_command_rules: Vec<DiscordCommandAdminRule>,
    slash_command_policy: DiscordSlashCommandPolicy,
}

struct DiscordChannelModeRequest {
    runtime: DiscordRuntimeLaunchConfig,
    acl: DiscordAclLaunchConfig,
}

pub(super) async fn run_discord_channel_command(
    req: ChannelCommandRequest,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<()> {
    let runtime = resolve_discord_runtime_launch_config(req, runtime_settings)?;
    let acl = resolve_discord_acl_launch_config(runtime_settings)?;
    run_discord_channel_mode(DiscordChannelModeRequest { runtime, acl }, runtime_settings).await
}

fn resolve_discord_runtime_launch_config(
    req: ChannelCommandRequest,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<DiscordRuntimeLaunchConfig> {
    let ChannelCommandRequest {
        bot_token,
        tool_config,
        session_partition,
        inbound_queue_capacity,
        turn_timeout_secs,
        discord_runtime_mode,
        ..
    } = req;

    let bot_token = bot_token
        .or_else(|| env_non_empty!("DISCORD_BOT_TOKEN"))
        .ok_or_else(|| anyhow::anyhow!("--bot-token or DISCORD_BOT_TOKEN required"))?;
    let ResolvedDiscordRuntimeConfig {
        runtime_config,
        runtime_mode,
    } = resolve_discord_runtime_config(
        runtime_settings,
        session_partition,
        inbound_queue_capacity,
        turn_timeout_secs,
        discord_runtime_mode,
    )?;
    let ingress_bind = resolve_string(
        None,
        "XIUXIAN_DAOCHANG_DISCORD_INGRESS_BIND",
        runtime_settings.discord.ingress_bind.as_deref(),
        DISCORD_DEFAULT_INGRESS_BIND,
    );
    let ingress_path = resolve_string(
        None,
        "XIUXIAN_DAOCHANG_DISCORD_INGRESS_PATH",
        runtime_settings.discord.ingress_path.as_deref(),
        DISCORD_DEFAULT_INGRESS_PATH,
    );
    let ingress_secret_token = env_non_empty!("XIUXIAN_DAOCHANG_DISCORD_INGRESS_SECRET_TOKEN")
        .or_else(|| runtime_settings.discord.ingress_secret_token.clone())
        .and_then(|secret| normalize_non_empty_secret(&secret));

    Ok(DiscordRuntimeLaunchConfig {
        bot_token,
        tool_config_path: tool_config,
        runtime_mode,
        runtime_config,
        ingress_bind,
        ingress_path,
        ingress_secret_token,
    })
}

fn resolve_discord_runtime_config(
    runtime_settings: &RuntimeSettings,
    session_partition: Option<String>,
    inbound_queue_capacity: Option<usize>,
    turn_timeout_secs: Option<u64>,
    discord_runtime_mode: Option<DiscordRuntimeMode>,
) -> anyhow::Result<ResolvedDiscordRuntimeConfig> {
    let raw_partition = resolve_string(
        session_partition,
        "XIUXIAN_DAOCHANG_DISCORD_SESSION_PARTITION",
        runtime_settings.discord.session_partition.as_deref(),
        "guild_channel_user",
    );
    let session_partition = raw_partition
        .parse::<DiscordSessionPartition>()
        .map_err(|_| anyhow::anyhow!("invalid discord session partition mode: {raw_partition}"))?;
    let runtime_mode = resolve_discord_runtime_mode(
        discord_runtime_mode,
        runtime_settings.discord.runtime_mode.as_deref(),
    );
    let inbound_queue_capacity = resolve_positive_usize(
        inbound_queue_capacity,
        "XIUXIAN_DAOCHANG_DISCORD_INBOUND_QUEUE_CAPACITY",
        runtime_settings.discord.inbound_queue_capacity,
        DISCORD_DEFAULT_INBOUND_QUEUE_CAPACITY,
    );
    let turn_timeout_secs = resolve_positive_u64(
        turn_timeout_secs,
        "XIUXIAN_DAOCHANG_DISCORD_TURN_TIMEOUT_SECS",
        runtime_settings.discord.turn_timeout_secs,
        DISCORD_DEFAULT_TURN_TIMEOUT_SECS,
    );
    let foreground_max_in_flight_messages = resolve_positive_usize(
        None,
        "XIUXIAN_DAOCHANG_DISCORD_FOREGROUND_MAX_IN_FLIGHT_MESSAGES",
        runtime_settings.discord.foreground_max_in_flight_messages,
        DISCORD_DEFAULT_FOREGROUND_MAX_IN_FLIGHT_MESSAGES,
    );
    let require_mention = resolve_bool(
        None,
        "XIUXIAN_DAOCHANG_DISCORD_REQUIRE_MENTION",
        runtime_settings.discord.require_mention,
        false,
    );
    let require_mention_persist = resolve_bool(
        None,
        "XIUXIAN_DAOCHANG_DISCORD_REQUIRE_MENTION_PERSIST",
        runtime_settings.discord.require_mention_persist,
        false,
    );

    Ok(ResolvedDiscordRuntimeConfig {
        runtime_mode,
        runtime_config: DiscordRuntimeConfig {
            session_partition,
            require_mention,
            require_mention_persist,
            mention_overrides: resolve_discord_mention_overrides(runtime_settings),
            inbound_queue_capacity,
            turn_timeout_secs,
            foreground_max_in_flight_messages,
            foreground_queue_mode: resolve_foreground_queue_mode(
                "XIUXIAN_DAOCHANG_DISCORD_FOREGROUND_QUEUE_MODE",
                runtime_settings.discord.foreground_queue_mode.as_deref(),
                ForegroundQueueMode::Queue,
            ),
        },
    })
}

fn resolve_discord_mention_overrides(runtime_settings: &RuntimeSettings) -> HashMap<String, bool> {
    runtime_settings
        .discord
        .channels
        .clone()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(recipient, settings)| {
            settings
                .require_mention
                .map(|value| (recipient.trim().to_string(), value))
        })
        .filter(|(recipient, _)| !recipient.is_empty())
        .collect::<HashMap<_, _>>()
}

fn resolve_discord_acl_launch_config(
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<DiscordAclLaunchConfig> {
    let acl_overrides = build_discord_acl_overrides(runtime_settings)?;
    let allowed_users = acl_overrides.allowed_users;
    let allowed_guilds = acl_overrides.allowed_guilds;
    let admin_users = acl_overrides
        .admin_users
        .unwrap_or_else(|| allowed_users.clone());
    let control_command_allow_from = acl_overrides.control_command_allow_from;
    let slash_global_allow_entries = acl_overrides.slash_command_allow_from;
    let slash_command_policy = DiscordSlashCommandPolicy {
        global: slash_global_allow_entries.clone(),
        session_status: acl_overrides.slash_session_status_allow_from,
        session_budget: acl_overrides.slash_session_budget_allow_from,
        session_memory: acl_overrides.slash_session_memory_allow_from,
        session_feedback: acl_overrides.slash_session_feedback_allow_from,
        job_status: acl_overrides.slash_job_allow_from,
        jobs_summary: acl_overrides.slash_jobs_allow_from,
        background_submit: acl_overrides.slash_bg_allow_from,
    };
    log_control_command_allow_override("discord", control_command_allow_from.as_deref());
    log_slash_command_allow_override("discord", slash_global_allow_entries.as_deref());

    Ok(DiscordAclLaunchConfig {
        allowed_users,
        allowed_guilds,
        admin_users,
        control_command_allow_from,
        control_command_rules: acl_overrides.control_command_rules,
        slash_command_policy,
    })
}

fn normalize_non_empty_secret(secret: &str) -> Option<String> {
    let trimmed = secret.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn resolve_foreground_queue_mode(
    env_name: &str,
    settings_value: Option<&str>,
    default: ForegroundQueueMode,
) -> ForegroundQueueMode {
    if let Some(raw) = env_non_empty!(env_name) {
        if let Some(mode) = ForegroundQueueMode::parse(raw.as_str()) {
            return mode;
        }
        tracing::warn!(
            env_var = %env_name,
            value = %raw,
            "invalid foreground queue mode env value; using settings/default"
        );
    }
    if let Some(raw) = settings_value {
        if let Some(mode) = ForegroundQueueMode::parse(raw) {
            return mode;
        }
        tracing::warn!(
            setting = "discord.foreground_queue_mode",
            value = %raw,
            "invalid discord foreground queue mode in settings; using default"
        );
    }
    default
}

async fn run_discord_channel_mode(
    request: DiscordChannelModeRequest,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<()> {
    let DiscordChannelModeRequest { runtime, acl } = request;
    let DiscordRuntimeLaunchConfig {
        bot_token,
        tool_config_path,
        runtime_mode,
        runtime_config,
        ingress_bind,
        ingress_path,
        ingress_secret_token,
    } = runtime;
    let DiscordAclLaunchConfig {
        allowed_users,
        allowed_guilds,
        admin_users,
        control_command_allow_from,
        control_command_rules,
        slash_command_policy,
    } = acl;

    let effective_runtime_settings = apply_channel_embedding_memory_guard(runtime_settings);
    let agent = Arc::new(build_agent(&tool_config_path, &effective_runtime_settings).await?);
    let control_command_policy = DiscordControlCommandPolicy::new(
        admin_users,
        control_command_allow_from,
        control_command_rules,
    )
    .with_slash_command_policy(slash_command_policy);

    if allowed_users.is_empty() && allowed_guilds.is_empty() {
        tracing::warn!(
            "Discord ACL allowlist is empty; all inbound will be rejected. \
             Configure `discord.acl.allow.users` or `discord.acl.allow.guilds` to allow traffic."
        );
    }

    match runtime_mode {
        DiscordRuntimeMode::Gateway => {
            run_discord_gateway(
                Arc::clone(&agent),
                bot_token,
                allowed_users,
                allowed_guilds,
                control_command_policy,
                runtime_config,
            )
            .await
        }
        DiscordRuntimeMode::Ingress => {
            run_discord_ingress(
                Arc::clone(&agent),
                DiscordIngressRunRequest {
                    bot_token,
                    allowed_users,
                    allowed_guilds,
                    control_command_policy,
                    bind_addr: ingress_bind,
                    ingress_path,
                    secret_token: ingress_secret_token,
                },
                runtime_config,
            )
            .await
        }
    }
}
