//! Discord ingress runtime server and foreground dispatch.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

use crate::agent::Agent;
use crate::channels::discord::channel::DiscordControlCommandPolicy;
use crate::channels::discord::runtime::DiscordRuntimeConfig;
use crate::channels::discord::runtime::{
    DiscordForegroundRuntime, build_foreground_runtime, snapshot_interval_from_env,
};
use crate::channels::discord::runtime::{
    DiscordIngressApp, DiscordIngressBuildRequest,
    build_discord_ingress_app_with_partition_and_control_command_policy,
};
use crate::channels::{Channel, ChannelMessage};

use super::loop_control::drive_ingress_runtime_loop;

/// Parameters to run Discord HTTP ingress runtime.
#[derive(Debug)]
pub struct DiscordIngressRunRequest {
    /// Bot token used by outbound Discord API calls.
    pub bot_token: String,
    /// Optional allowlist of user ids.
    pub allowed_users: Vec<String>,
    /// Optional allowlist of guild ids.
    pub allowed_guilds: Vec<String>,
    /// Policy for control and slash managed commands.
    pub control_command_policy: DiscordControlCommandPolicy,
    /// TCP address for ingress listener.
    pub bind_addr: String,
    /// HTTP path for ingress endpoint.
    pub ingress_path: String,
    /// Optional shared secret token for ingress validation.
    pub secret_token: Option<String>,
}

/// Run Discord channel via HTTP ingress endpoint.
///
/// # Errors
/// Returns an error when channel/runtime initialization fails.
pub async fn run_discord_ingress(
    agent: Arc<Agent>,
    request: DiscordIngressRunRequest,
    runtime_config: DiscordRuntimeConfig,
) -> Result<()> {
    let DiscordIngressRunRequest {
        bot_token,
        allowed_users,
        allowed_guilds,
        control_command_policy,
        bind_addr,
        ingress_path,
        secret_token,
    } = request;
    let DiscordRuntimeConfig {
        session_partition,
        require_mention,
        require_mention_persist,
        mention_overrides,
        inbound_queue_capacity,
        turn_timeout_secs,
        foreground_max_in_flight_messages,
        foreground_queue_mode,
    } = runtime_config;

    let (tx, mut inbound_rx) = mpsc::channel::<ChannelMessage>(inbound_queue_capacity);
    let inbound_snapshot_tx = tx.clone();
    let DiscordIngressApp { app, channel, path } = build_ingress_app(
        DiscordIngressBuildRequest {
            bot_token,
            allowed_users,
            allowed_guilds,
            control_command_policy,
            ingress_path,
            secret_token,
            session_partition,
            tx,
        },
        require_mention,
        require_mention_persist,
        mention_overrides,
    )
    .await?;
    let channel_for_send: Arc<dyn Channel> = channel.clone();
    let (mut runtime, mut completion_rx) = build_ingress_foreground_runtime(
        agent,
        channel_for_send,
        turn_timeout_secs,
        foreground_max_in_flight_messages,
        foreground_queue_mode,
    );
    let mut snapshot_tick = build_snapshot_tick();
    if let Some(interval) = snapshot_tick.as_mut() {
        let _ = interval.tick().await;
    }
    let listener = TcpListener::bind(&bind_addr).await?;

    let (shutdown_tx, mut ingress_server) = spawn_ingress_server(listener, app);

    print_ingress_runtime_banner(
        &bind_addr,
        &path,
        channel.session_partition(),
        inbound_queue_capacity,
        foreground_max_in_flight_messages,
        turn_timeout_secs,
        foreground_queue_mode,
    );

    drive_ingress_runtime_loop(
        &mut runtime,
        &mut inbound_rx,
        &mut completion_rx,
        &inbound_snapshot_tx,
        inbound_queue_capacity,
        &mut snapshot_tick,
        &mut ingress_server,
    )
    .await;

    runtime.abort_and_drain_foreground_tasks().await;

    let _ = shutdown_tx.send(());
    Ok(())
}

fn build_snapshot_tick() -> Option<tokio::time::Interval> {
    snapshot_interval_from_env().map(|period| {
        let mut interval = tokio::time::interval(period);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        interval
    })
}

async fn build_ingress_app(
    request: DiscordIngressBuildRequest,
    require_mention: bool,
    require_mention_persist: bool,
    mention_overrides: HashMap<String, bool>,
) -> Result<DiscordIngressApp> {
    let ingress = build_discord_ingress_app_with_partition_and_control_command_policy(request)?;
    ingress.channel.configure_mention_policy(
        require_mention,
        require_mention_persist,
        mention_overrides,
    );
    ingress.channel.hydrate_bot_identity().await?;
    Ok(ingress)
}

fn build_ingress_foreground_runtime(
    agent: Arc<Agent>,
    channel_for_send: Arc<dyn Channel>,
    turn_timeout_secs: u64,
    foreground_max_in_flight_messages: usize,
    foreground_queue_mode: crate::channels::managed_runtime::ForegroundQueueMode,
) -> (
    DiscordForegroundRuntime,
    mpsc::Receiver<crate::jobs::JobCompletion>,
) {
    build_foreground_runtime(
        agent,
        channel_for_send,
        turn_timeout_secs,
        foreground_max_in_flight_messages,
        foreground_queue_mode,
    )
}

fn spawn_ingress_server(
    listener: TcpListener,
    app: Router,
) -> (
    tokio::sync::oneshot::Sender<()>,
    tokio::task::JoinHandle<std::io::Result<()>>,
) {
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let ingress_server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
    });
    (shutdown_tx, ingress_server)
}

fn print_ingress_runtime_banner(
    bind_addr: &str,
    path: &str,
    session_partition: impl std::fmt::Display,
    inbound_queue_capacity: usize,
    foreground_max_in_flight_messages: usize,
    turn_timeout_secs: u64,
    foreground_queue_mode: impl std::fmt::Display,
) {
    println!("Discord ingress listening on {bind_addr}{path} (Ctrl+C to stop)");
    println!("Discord session partition: {session_partition}");
    println!(
        "Discord foreground config: inbound_queue={inbound_queue_capacity} max_in_flight={foreground_max_in_flight_messages} timeout={turn_timeout_secs}s queue_mode={foreground_queue_mode}"
    );
    println!("Background commands: /bg <prompt>, /job <id> [json], /jobs [json]");
    println!(
        "Session commands: /help [json], /session [json], /session budget [json], /session memory [json], /session feedback up|down [json], /session mention [status|on|off|inherit] [json], /session partition [mode|on|off] [json], /session admin [list|set|add|remove|clear] [json], /session inject [status|clear|<qa>...</qa>] [json], /feedback up|down [json], /reset, /clear, /resume, /resume drop, /stop"
    );
}
