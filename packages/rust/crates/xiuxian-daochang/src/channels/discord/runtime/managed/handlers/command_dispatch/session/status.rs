use std::sync::Arc;

use crate::agent::Agent;
use crate::channels::managed_commands::SLASH_SCOPE_SESSION_STATUS;
use crate::channels::traits::{Channel, ChannelMessage};

use crate::channels::discord::runtime::managed::handlers::auth::ensure_slash_command_authorized;
use crate::channels::discord::runtime::managed::handlers::events::{
    EVENT_DISCORD_COMMAND_SESSION_STATUS_JSON_REPLIED, EVENT_DISCORD_COMMAND_SESSION_STATUS_REPLIED,
};
use crate::channels::discord::runtime::managed::handlers::send::send_response;
use crate::channels::discord::runtime::managed::parsing::CommandOutputFormat;
use crate::channels::discord::runtime::managed::replies::{
    format_command_error_json, format_session_context_snapshot,
    format_session_context_snapshot_json,
};

pub(in super::super) async fn handle_session_status(
    agent: &Arc<Agent>,
    channel: &Arc<dyn Channel>,
    msg: &ChannelMessage,
    session_id: &str,
    format: CommandOutputFormat,
) {
    if !ensure_slash_command_authorized(channel, msg, SLASH_SCOPE_SESSION_STATUS, "/session").await
    {
        return;
    }
    let command_event = if format.is_json() {
        EVENT_DISCORD_COMMAND_SESSION_STATUS_JSON_REPLIED
    } else {
        EVENT_DISCORD_COMMAND_SESSION_STATUS_REPLIED
    };
    let admission_status = agent.downstream_admission_runtime_snapshot();
    let response = match (
        agent.inspect_context_window(session_id).await,
        agent.peek_context_window_backup(session_id).await,
    ) {
        (Ok(active), Ok(snapshot)) => {
            let partition_mode = channel
                .session_partition_mode()
                .unwrap_or_else(|| "unknown".to_string());
            if format.is_json() {
                format_session_context_snapshot_json(
                    session_id,
                    &msg.session_key,
                    &partition_mode,
                    active,
                    snapshot,
                    admission_status,
                )
            } else {
                format_session_context_snapshot(
                    session_id,
                    &msg.session_key,
                    &partition_mode,
                    active,
                    snapshot,
                    admission_status,
                )
            }
        }
        (Err(error), _) if format.is_json() => {
            format_command_error_json("session_context_status", &error.to_string())
        }
        (_, Err(error)) if format.is_json() => {
            format_command_error_json("session_context_status", &error.to_string())
        }
        (Err(error), _) => format!("Failed to inspect active session context: {error}"),
        (_, Err(error)) => format!("Failed to inspect saved session snapshot: {error}"),
    };
    send_response(channel, &msg.recipient, response, msg, command_event).await;
}
