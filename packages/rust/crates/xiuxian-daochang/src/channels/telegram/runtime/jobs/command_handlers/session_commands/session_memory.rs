use std::sync::Arc;

use crate::agent::Agent;
use crate::channels::managed_commands::SLASH_SCOPE_SESSION_MEMORY as TELEGRAM_SLASH_SCOPE_SESSION_MEMORY;
use crate::channels::telegram::commands::parse_session_context_memory_command;
use crate::channels::traits::{Channel, ChannelMessage};

use super::{
    EVENT_TELEGRAM_COMMAND_SESSION_MEMORY_JSON_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_MEMORY_REPLIED,
};
use crate::channels::telegram::runtime::jobs::command_handlers::slash_acl::ensure_slash_command_authorized;
use crate::channels::telegram::runtime::jobs::observability::send_with_observability;
use crate::channels::telegram::runtime::jobs::replies::{
    format_memory_recall_compact_not_found, format_memory_recall_compact_snapshot,
    format_memory_recall_not_found, format_memory_recall_not_found_json,
    format_memory_recall_snapshot, format_memory_recall_snapshot_json,
};

pub(in crate::channels::telegram::runtime::jobs) async fn try_handle_session_memory_command(
    msg: &ChannelMessage,
    channel: &Arc<dyn Channel>,
    agent: &Arc<Agent>,
    session_id: &str,
) -> bool {
    let Some(format) = parse_session_context_memory_command(&msg.content) else {
        return false;
    };

    if !ensure_slash_command_authorized(
        channel,
        msg,
        TELEGRAM_SLASH_SCOPE_SESSION_MEMORY,
        "/session memory",
    )
    .await
    {
        return true;
    }

    let command_event = if format.is_json() {
        EVENT_TELEGRAM_COMMAND_SESSION_MEMORY_JSON_REPLIED
    } else {
        EVENT_TELEGRAM_COMMAND_SESSION_MEMORY_REPLIED
    };
    let runtime_status = agent.inspect_memory_runtime_status();
    let admission_status = agent.downstream_admission_runtime_snapshot();
    let metrics = agent.inspect_memory_recall_metrics().await;
    let response = match agent.inspect_memory_recall_snapshot(session_id).await {
        Some(snapshot) if format.is_json() => format_memory_recall_snapshot_json(
            snapshot,
            metrics,
            &runtime_status,
            admission_status,
            session_id,
        ),
        Some(snapshot) if channel.name() == "telegram" => format_memory_recall_compact_snapshot(
            snapshot,
            &runtime_status,
            admission_status,
            session_id,
        ),
        Some(snapshot) => format_memory_recall_snapshot(
            snapshot,
            metrics,
            runtime_status,
            admission_status,
            session_id,
        ),
        None if format.is_json() => format_memory_recall_not_found_json(
            metrics,
            &runtime_status,
            admission_status,
            session_id,
        ),
        None if channel.name() == "telegram" => {
            format_memory_recall_compact_not_found(&runtime_status, admission_status, session_id)
        }
        None => format_memory_recall_not_found(runtime_status, admission_status, session_id),
    };
    send_with_observability(
        channel,
        &response,
        &msg.recipient,
        "Failed to send session memory response",
        Some(command_event),
        Some(&msg.session_key),
    )
    .await;
    true
}
