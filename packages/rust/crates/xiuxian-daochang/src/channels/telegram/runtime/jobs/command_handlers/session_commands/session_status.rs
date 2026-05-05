use std::sync::Arc;

use crate::agent::Agent;
use crate::channels::managed_commands::SLASH_SCOPE_SESSION_STATUS as TELEGRAM_SLASH_SCOPE_SESSION_STATUS;
use crate::channels::telegram::commands::parse_session_context_status_command;
use crate::channels::traits::{Channel, ChannelMessage};

use super::{
    EVENT_TELEGRAM_COMMAND_SESSION_STATUS_JSON_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_STATUS_REPLIED,
};
use crate::channels::telegram::runtime::jobs::command_handlers::slash_acl::ensure_slash_command_authorized;
use crate::channels::telegram::runtime::jobs::observability::send_with_observability;
use crate::channels::telegram::runtime::jobs::replies::{
    format_command_error_json, format_session_context_snapshot,
    format_session_context_snapshot_json,
};

pub(in crate::channels::telegram::runtime::jobs) async fn try_handle_session_status_command(
    msg: &ChannelMessage,
    channel: &Arc<dyn Channel>,
    agent: &Arc<Agent>,
    session_id: &str,
) -> bool {
    let Some(format) = parse_session_context_status_command(&msg.content) else {
        return false;
    };

    if !ensure_slash_command_authorized(
        channel,
        msg,
        TELEGRAM_SLASH_SCOPE_SESSION_STATUS,
        "/session",
    )
    .await
    {
        return true;
    }

    let command_event = if format.is_json() {
        EVENT_TELEGRAM_COMMAND_SESSION_STATUS_JSON_REPLIED
    } else {
        EVENT_TELEGRAM_COMMAND_SESSION_STATUS_REPLIED
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
    send_with_observability(
        channel,
        &response,
        &msg.recipient,
        "Failed to send session status response",
        Some(command_event),
        Some(&msg.session_key),
    )
    .await;
    true
}
