//! Discord foreground message dispatch handler.

use std::sync::Arc;

use crate::agent::Agent;
use crate::channels::managed_commands::{
    detect_managed_control_command, detect_managed_slash_command,
};
use crate::channels::managed_runtime::ForegroundQueueMode;
use crate::channels::managed_runtime::parsing::is_stop_command;
use crate::channels::managed_runtime::{
    ForegroundTurnOutcome, build_session_id, compose_turn_content,
};
use crate::channels::{Channel, ChannelMessage};
use crate::jobs::JobManager;

use super::generation::begin_active_generation;
use super::stop::try_handle_stop_command;
use super::support::{log_inbound_user_message, log_preempted_turn};
use super::turn::{
    ForegroundTurnInput, render_foreground_turn_reply, run_foreground_turn_with_typing,
};
use crate::channels::discord::runtime::ForegroundInterruptController;
use crate::channels::discord::runtime::managed::handle_inbound_managed_command;

pub(in crate::channels::discord::runtime) async fn process_discord_message(
    agent: Arc<Agent>,
    channel: Arc<dyn Channel>,
    msg: ChannelMessage,
    job_manager: &Arc<JobManager>,
    turn_timeout_secs: u64,
) {
    let interrupt_controller = ForegroundInterruptController::default();
    process_discord_message_with_interrupt(
        agent,
        channel,
        msg,
        job_manager,
        turn_timeout_secs,
        ForegroundQueueMode::Queue,
        &interrupt_controller,
    )
    .await;
}

pub(in crate::channels::discord::runtime) async fn process_discord_message_with_interrupt(
    agent: Arc<Agent>,
    channel: Arc<dyn Channel>,
    msg: ChannelMessage,
    job_manager: &Arc<JobManager>,
    turn_timeout_secs: u64,
    foreground_queue_mode: ForegroundQueueMode,
    interrupt_controller: &ForegroundInterruptController,
) {
    let session_id = build_session_id(&msg.channel, &msg.session_key);

    if is_stop_command(&msg.content)
        && try_handle_stop_command(
            agent.as_ref(),
            channel.as_ref(),
            &msg,
            &session_id,
            interrupt_controller,
        )
        .await
    {
        return;
    }

    if let Some(control_command) = detect_managed_control_command(&msg.content) {
        tracing::debug!(
            command = control_command.canonical_command(),
            "discord managed control command detected"
        );
    }
    if let Some(slash_command) = detect_managed_slash_command(&msg.content) {
        tracing::debug!(
            command = slash_command.canonical_command(),
            scope = slash_command.scope(),
            "discord managed slash command detected"
        );
    }

    if handle_inbound_managed_command(&agent, &channel, &msg, job_manager).await {
        return;
    }

    if foreground_queue_mode.should_interrupt_on_new_message() {
        log_preempted_turn(interrupt_controller, &session_id, &msg);
    }
    log_inbound_user_message(&msg);
    let (interrupt_rx, _active_generation_guard, interrupt_generation) =
        begin_active_generation(interrupt_controller, &session_id);
    let turn_content = compose_turn_content(&msg);
    let turn_input = ForegroundTurnInput {
        recipient: &msg.recipient,
        session_id: &session_id,
        content: turn_content.as_str(),
        turn_timeout_secs,
        interrupt_rx,
        interrupt_generation,
    };
    let result = run_foreground_turn_with_typing(channel.as_ref(), agent.clone(), turn_input).await;
    let reply = match result {
        ForegroundTurnOutcome::TimedOut { .. } => Some(
            queue_timed_out_foreground_turn(
                job_manager,
                &msg,
                &session_id,
                &turn_content,
                turn_timeout_secs,
            )
            .await,
        ),
        other => render_foreground_turn_reply(other, &msg, turn_timeout_secs),
    };
    if let Some(reply) = reply {
        super::turn::send_discord_reply(channel.as_ref(), &msg, &reply).await;
    }
}

async fn queue_timed_out_foreground_turn(
    job_manager: &Arc<JobManager>,
    msg: &ChannelMessage,
    session_id: &str,
    prompt: &str,
    timeout_secs: u64,
) -> String {
    match job_manager
        .submit(session_id, msg.recipient.clone(), prompt.to_string())
        .await
    {
        Ok(job_id) => {
            tracing::warn!(
                event = "discord.foreground.turn.timeout_background_submit",
                session_key = %msg.session_key,
                channel = %msg.channel,
                recipient = %msg.recipient,
                sender = %msg.sender,
                timeout_secs,
                job_id = %job_id,
                "discord foreground turn timed out and was requeued as a background job"
            );
            format!(
                "Still working on that. Moved it to background job `{job_id}` and will post the result here when it's ready.\nUse `/job {job_id}` for status."
            )
        }
        Err(error) => {
            tracing::warn!(
                event = "discord.foreground.turn.timeout_background_submit_failed",
                session_key = %msg.session_key,
                channel = %msg.channel,
                recipient = %msg.recipient,
                sender = %msg.sender,
                timeout_secs,
                error = %error,
                "discord foreground turn timed out and background submission failed"
            );
            format!(
                "Request timed out after {timeout_secs}s.\nBackground queue submission failed: {error}"
            )
        }
    }
}

pub(in crate::channels::discord::runtime) fn test_interrupted_reply_is_suppressed(
    _msg: &ChannelMessage,
    _turn_timeout_secs: u64,
) -> bool {
    true
}
