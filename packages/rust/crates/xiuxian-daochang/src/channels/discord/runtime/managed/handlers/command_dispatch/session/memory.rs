use std::sync::Arc;

use crate::agent::Agent;
use crate::channels::managed_commands::SLASH_SCOPE_SESSION_MEMORY;
use crate::channels::traits::{Channel, ChannelMessage};

use crate::channels::discord::runtime::managed::handlers::auth::ensure_slash_command_authorized;
use crate::channels::discord::runtime::managed::handlers::events::{
    EVENT_DISCORD_COMMAND_SESSION_MEMORY_JSON_REPLIED, EVENT_DISCORD_COMMAND_SESSION_MEMORY_REPLIED,
};
use crate::channels::discord::runtime::managed::handlers::send::send_response;
use crate::channels::discord::runtime::managed::parsing::CommandOutputFormat;
use crate::channels::discord::runtime::managed::replies::{
    format_memory_recall_not_found, format_memory_recall_not_found_json,
    format_memory_recall_snapshot, format_memory_recall_snapshot_json,
};

pub(in super::super) async fn handle_session_memory(
    agent: &Arc<Agent>,
    channel: &Arc<dyn Channel>,
    msg: &ChannelMessage,
    session_id: &str,
    format: CommandOutputFormat,
) {
    if !ensure_slash_command_authorized(channel, msg, SLASH_SCOPE_SESSION_MEMORY, "/session memory")
        .await
    {
        return;
    }
    let command_event = if format.is_json() {
        EVENT_DISCORD_COMMAND_SESSION_MEMORY_JSON_REPLIED
    } else {
        EVENT_DISCORD_COMMAND_SESSION_MEMORY_REPLIED
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
        None => format_memory_recall_not_found(runtime_status, admission_status, session_id),
    };
    send_response(channel, &msg.recipient, response, msg, command_event).await;
}
