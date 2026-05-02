use crate::agent::Agent;
use crate::channels::discord::runtime::ForegroundInterruptController;
use crate::channels::{Channel, ChannelMessage};

pub(super) async fn try_handle_stop_command(
    agent: &Agent,
    channel: &dyn Channel,
    msg: &ChannelMessage,
    session_id: &str,
    interrupt_controller: &ForegroundInterruptController,
) -> bool {
    let interrupted = interrupt_controller.interrupt(session_id);
    if interrupted
        && let Err(error) = agent
            .append_turn_for_session(
                session_id,
                "[control] /stop",
                "[system] Current foreground generation interrupted by user request.",
            )
            .await
    {
        tracing::warn!(
            session_id = %session_id,
            error = %error,
            "failed to persist discord stop-interrupted marker for session"
        );
    }

    let response = if interrupted {
        "Stop signal sent. Current foreground generation is being interrupted."
    } else {
        "No active foreground generation to stop in this session."
    };

    let event_name = if interrupted {
        "discord.command.session_stop.replied"
    } else {
        "discord.command.session_stop_idle.replied"
    };
    match channel.send(response, &msg.recipient).await {
        Ok(()) => {
            tracing::info!(
                event = event_name,
                session_key = %msg.session_key,
                channel = %msg.channel,
                recipient = %msg.recipient,
                sender = %msg.sender,
                "discord session stop command replied"
            );
        }
        Err(error) => {
            tracing::warn!(
                event = "discord.command.session_stop.reply_failed",
                session_key = %msg.session_key,
                channel = %msg.channel,
                recipient = %msg.recipient,
                sender = %msg.sender,
                error = %error,
                "discord failed to send stop reply"
            );
        }
    }
    true
}
