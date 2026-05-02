//! Command-line runtime entrypoint.

use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::cli::{Cli, Command};
use crate::nodes::{
    ChannelCommandRequest, ScheduleModeRequest, run_channel_command, run_embedding_warmup,
    run_gateway_mode, run_repl_mode, run_schedule_mode, run_stdio_mode,
};
use crate::{RuntimeSettings, load_runtime_settings, set_config_home_override};

/// Run the command-line application.
///
/// # Errors
///
/// Returns an error when the selected runtime mode fails to initialize or run.
pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if let Some(conf_dir) = cli.conf.clone() {
        set_config_home_override(conf_dir);
    }
    let runtime_settings = load_runtime_settings();

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("omni_agent=info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();

    run_command(cli.command, &runtime_settings).await
}

async fn run_command(command: Command, runtime_settings: &RuntimeSettings) -> anyhow::Result<()> {
    match command {
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
                runtime_settings,
            )
            .await
        }
        Command::Stdio {
            session_id,
            tool_config,
        } => run_stdio_mode(session_id, tool_config, runtime_settings).await,
        Command::Repl {
            query,
            session_id,
            tool_config,
        } => run_repl_mode(query, session_id, tool_config, runtime_settings).await,
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
                ScheduleModeRequest {
                    prompt,
                    interval_secs,
                    max_runs,
                    schedule_id,
                    session_prefix,
                    recipient,
                    wait_for_completion_secs,
                    tool_config_path: tool_config,
                },
                runtime_settings,
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
                runtime_settings,
            )
            .await
        }
        Command::EmbeddingWarmup { text, model } => {
            run_embedding_warmup(runtime_settings, text, model).await
        }
    }
}
