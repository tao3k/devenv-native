use super::preview::log_preview;
use crate::channels::ChannelMessage;
use crate::channels::discord::runtime::ForegroundInterruptController;

pub(super) fn log_inbound_user_message(msg: &ChannelMessage) {
    tracing::info!(
        event = "discord.foreground.turn.inbound",
        session_key = %msg.session_key,
        channel = %msg.channel,
        recipient = %msg.recipient,
        sender = %msg.sender,
        preview = %log_preview(&msg.content),
        "discord inbound user message queued for foreground turn"
    );
}

pub(super) fn log_preempted_turn(
    interrupt_controller: &ForegroundInterruptController,
    session_id: &str,
    msg: &ChannelMessage,
) {
    if interrupt_controller.interrupt(session_id) {
        tracing::info!(
            event = "discord.foreground.turn.preempted",
            session_key = %msg.session_key,
            channel = %msg.channel,
            recipient = %msg.recipient,
            sender = %msg.sender,
            "active foreground turn interrupted by newer inbound message"
        );
    }
}
