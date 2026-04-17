use std::sync::Arc;

use crate::agent::Agent;
use crate::channels::managed_commands::SLASH_SCOPE_SESSION_BUDGET as TELEGRAM_SLASH_SCOPE_SESSION_BUDGET;
use crate::channels::telegram::commands::parse_session_context_budget_command;
use crate::channels::traits::{Channel, ChannelMessage};

use super::super::super::observability::send_with_observability;
use super::super::super::replies::{
    format_context_budget_not_found_json, format_context_budget_snapshot,
    format_context_budget_snapshot_json,
};
use super::super::slash_acl::ensure_slash_command_authorized;
use super::{
    EVENT_TELEGRAM_COMMAND_SESSION_BUDGET_JSON_REPLIED,
    EVENT_TELEGRAM_COMMAND_SESSION_BUDGET_REPLIED,
};

pub(in crate::channels::telegram::runtime::jobs) async fn try_handle_session_budget_command(
    msg: &ChannelMessage,
    channel: &Arc<dyn Channel>,
    agent: &Arc<Agent>,
    session_id: &str,
) -> bool {
    let Some(format) = parse_session_context_budget_command(&msg.content) else {
        return false;
    };

    if !ensure_slash_command_authorized(
        channel,
        msg,
        TELEGRAM_SLASH_SCOPE_SESSION_BUDGET,
        "/session budget",
    )
    .await
    {
        return true;
    }

    let command_event = if format.is_json() {
        EVENT_TELEGRAM_COMMAND_SESSION_BUDGET_JSON_REPLIED
    } else {
        EVENT_TELEGRAM_COMMAND_SESSION_BUDGET_REPLIED
    };
    let response = match agent.inspect_context_budget_snapshot(session_id).await {
        Some(snapshot) if format.is_json() => format_context_budget_snapshot_json(&snapshot),
        Some(snapshot) => format_context_budget_snapshot(&snapshot),
        None if format.is_json() => format_context_budget_not_found_json(),
        None => {
            "No context budget snapshot found for this session yet.\nRun at least one normal turn first (non-command message)."
                .to_string()
        }
    };
    send_with_observability(
        channel,
        &response,
        &msg.recipient,
        "Failed to send session budget response",
        Some(command_event),
        Some(&msg.session_key),
    )
    .await;
    true
}
