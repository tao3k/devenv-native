#![recursion_limit = "256"]
//! omni-agent CLI: gateway, stdio, or repl mode.
//!
//! External tool servers are loaded from `.tool.json` by default.
//! Override with `--tool-config <path>`.
//!
//! Logging: set `RUST_LOG=omni_agent=info` (or `warn`, `debug`) to see agent logs on stderr.

mod agent_builder;
mod cli;
mod nodes;
mod resolve;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use xiuxian_daochang::{load_runtime_settings, set_config_home_override};

use crate::cli::{Cli, Command};
pub(crate) use crate::cli::{DiscordRuntimeMode, TelegramChannelMode, WebhookDedupBackendMode};
use crate::nodes::{
    ChannelCommandRequest, run_channel_command, run_embedding_warmup, run_gateway_mode,
    run_repl_mode, run_schedule_mode, run_stdio_mode,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if let Some(conf_dir) = cli.conf.clone() {
        set_config_home_override(conf_dir);
    }
    let runtime_settings = load_runtime_settings();

    // Initialize tracing: RUST_LOG overrides; --verbose on channel => debug; else info
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("omni_agent=info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();

    match cli.command {
        Command::Gateway {
            bind,
            turn_timeout,
            max_concurrent,
            tool_config,
        } => {
            run_gateway_mode(
                bind,
                turn_timeout,
                max_concurrent,
                tool_config,
                &runtime_settings,
            )
            .await
        }
        Command::Stdio {
            session_id,
            tool_config,
        } => run_stdio_mode(session_id, tool_config, &runtime_settings).await,
        Command::Repl {
            query,
            session_id,
            tool_config,
        } => run_repl_mode(query, session_id, tool_config, &runtime_settings).await,
        Command::Schedule {
            prompt,
            interval_secs,
            max_runs,
            schedule_id,
            session_prefix,
            recipient,
            wait_for_completion_secs,
            tool_config,
        } => {
            run_schedule_mode(
                prompt,
                interval_secs,
                max_runs,
                schedule_id,
                session_prefix,
                recipient,
                wait_for_completion_secs,
                tool_config,
                &runtime_settings,
            )
            .await
        }
        Command::Channel {
            provider,
            bot_token,
            tool_config,
            mode,
            webhook_bind,
            webhook_path,
            webhook_secret_token,
            session_partition,
            inbound_queue_capacity,
            turn_timeout_secs,
            discord_runtime_mode,
            webhook_dedup_backend,
            valkey_url,
            webhook_dedup_ttl_secs,
            webhook_dedup_key_prefix,
        } => {
            run_channel_command(
                ChannelCommandRequest {
                    provider,
                    bot_token,
                    tool_config,
                    mode,
                    webhook_bind,
                    webhook_path,
                    webhook_secret_token,
                    session_partition,
                    inbound_queue_capacity,
                    turn_timeout_secs,
                    discord_runtime_mode,
                    webhook_dedup_backend,
                    valkey_url,
                    webhook_dedup_ttl_secs,
                    webhook_dedup_key_prefix,
                },
                &runtime_settings,
            )
            .await
        }
        Command::EmbeddingWarmup { text, model } => {
            run_embedding_warmup(&runtime_settings, text, model).await
        }
    }
}
