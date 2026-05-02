//! Telegram polling runtime event loop.

use std::sync::Arc;

use anyhow::Result;

use crate::agent::Agent;
use crate::channels::telegram::TelegramCommandAdminRule;
use crate::channels::telegram::TelegramControlCommandPolicy;
use crate::channels::telegram::runtime::console::{
    print_foreground_config, print_managed_commands_help,
};
use crate::channels::telegram::runtime::dispatch::start_telegram_runtime;
use crate::channels::telegram::runtime::run_polling::channel_listener;
use crate::channels::telegram::runtime::run_polling::loop_control;
use crate::channels::telegram::runtime_config::TelegramRuntimeConfig;

/// Run Telegram channel via long polling.
///
/// # Errors
/// Returns an error when runtime initialization or channel startup fails.
pub async fn run_telegram(
    agent: Arc<Agent>,
    bot_token: String,
    allowed_users: Vec<String>,
    allowed_groups: Vec<String>,
    admin_users: Vec<String>,
    control_command_allow_from: Option<Vec<String>>,
    control_command_rules: Vec<TelegramCommandAdminRule>,
) -> Result<()> {
    run_telegram_with_control_command_policy(
        agent,
        bot_token,
        allowed_users,
        allowed_groups,
        TelegramControlCommandPolicy::new(
            admin_users,
            control_command_allow_from,
            control_command_rules,
        ),
    )
    .await
}

/// Run Telegram channel via long polling with structured control-command policy.
///
/// # Errors
/// Returns an error when runtime initialization or channel startup fails.
pub async fn run_telegram_with_control_command_policy(
    agent: Arc<Agent>,
    bot_token: String,
    allowed_users: Vec<String>,
    allowed_groups: Vec<String>,
    control_command_policy: TelegramControlCommandPolicy,
) -> Result<()> {
    let runtime_config = TelegramRuntimeConfig::from_env();
    let (channel, channel_for_send, inbound_tx, mut inbound_rx, listener) =
        channel_listener::start_polling_listener(
            bot_token,
            allowed_users,
            allowed_groups,
            control_command_policy,
            runtime_config.inbound_queue_capacity,
        )?;

    let (
        session_gate_backend,
        foreground_tx,
        interrupt_controller,
        foreground_dispatcher,
        job_manager,
        mut completion_rx,
    ) = start_telegram_runtime(Arc::clone(&agent), &channel_for_send, runtime_config)?;

    println!("Telegram channel listening... (polling, Ctrl+C to stop)");
    println!("Session partition: {}", channel.session_partition());
    print_foreground_config(&runtime_config, &session_gate_backend);
    print_managed_commands_help();

    loop_control::run_polling_event_loop(
        loop_control::PollingLoopReceivers {
            inbound_rx: &mut inbound_rx,
            completion_rx: &mut completion_rx,
        },
        loop_control::PollingLoopContext {
            inbound_tx: &inbound_tx,
            channel_for_send: &channel_for_send,
            foreground_tx: &foreground_tx,
            interrupt_controller: &interrupt_controller,
            job_manager: &job_manager,
            agent: &agent,
            runtime_config,
        },
    )
    .await;

    drop(foreground_tx);
    foreground_dispatcher.abort();
    listener.abort();
    Ok(())
}
