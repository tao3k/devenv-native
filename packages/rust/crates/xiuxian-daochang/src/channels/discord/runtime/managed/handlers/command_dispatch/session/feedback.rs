use std::sync::Arc;

use crate::agent::{Agent, SessionRecallFeedbackDirection};
use crate::channels::managed_commands::SLASH_SCOPE_SESSION_FEEDBACK;
use crate::channels::traits::{Channel, ChannelMessage};

use crate::channels::discord::runtime::managed::handlers::auth::ensure_slash_command_authorized;
use crate::channels::discord::runtime::managed::handlers::events::{
    EVENT_DISCORD_COMMAND_SESSION_FEEDBACK_JSON_REPLIED,
    EVENT_DISCORD_COMMAND_SESSION_FEEDBACK_REPLIED,
};
use crate::channels::discord::runtime::managed::handlers::send::send_response;
use crate::channels::discord::runtime::managed::parsing::{
    FeedbackDirection, SessionFeedbackCommand,
};
use crate::channels::discord::runtime::managed::replies::{
    format_session_feedback, format_session_feedback_json, format_session_feedback_unavailable_json,
};

pub(in super::super) async fn handle_session_feedback(
    agent: &Arc<Agent>,
    channel: &Arc<dyn Channel>,
    msg: &ChannelMessage,
    session_id: &str,
    command: SessionFeedbackCommand,
) {
    if !ensure_slash_command_authorized(
        channel,
        msg,
        SLASH_SCOPE_SESSION_FEEDBACK,
        "/session feedback",
    )
    .await
    {
        return;
    }
    let command_event = if command.format.is_json() {
        EVENT_DISCORD_COMMAND_SESSION_FEEDBACK_JSON_REPLIED
    } else {
        EVENT_DISCORD_COMMAND_SESSION_FEEDBACK_REPLIED
    };
    let direction = match command.direction {
        FeedbackDirection::Up => SessionRecallFeedbackDirection::Up,
        FeedbackDirection::Down => SessionRecallFeedbackDirection::Down,
    };
    let response = match agent
        .apply_session_recall_feedback(session_id, direction)
        .await
    {
        Some(update) if command.format.is_json() => {
            format_session_feedback_json(direction, update.previous_bias, update.updated_bias)
        }
        Some(update) => {
            format_session_feedback(direction, update.previous_bias, update.updated_bias)
        }
        None if command.format.is_json() => format_session_feedback_unavailable_json(),
        None => "Session recall feedback is unavailable because memory is disabled.".to_string(),
    };
    send_response(channel, &msg.recipient, response, msg, command_event).await;
}
