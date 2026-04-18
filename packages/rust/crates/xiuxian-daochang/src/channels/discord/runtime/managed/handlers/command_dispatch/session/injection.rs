use std::sync::Arc;

use serde_json::json;

use crate::agent::Agent;
use crate::channels::discord::runtime::managed::handlers::auth::ensure_control_command_authorized;
use crate::channels::discord::runtime::managed::handlers::events::{
    EVENT_DISCORD_COMMAND_SESSION_INJECTION_JSON_REPLIED,
    EVENT_DISCORD_COMMAND_SESSION_INJECTION_REPLIED,
};
use crate::channels::discord::runtime::managed::handlers::send::send_response;
use crate::channels::discord::runtime::managed::replies::format_command_error_json;
use crate::channels::telegram::commands::{SessionInjectionAction, SessionInjectionCommand};
use crate::channels::traits::{Channel, ChannelMessage};

pub(in super::super) async fn handle_session_injection(
    agent: &Arc<Agent>,
    channel: &Arc<dyn Channel>,
    msg: &ChannelMessage,
    session_id: &str,
    command: SessionInjectionCommand,
) {
    if !ensure_control_command_authorized(channel, msg, "/session inject").await {
        return;
    }

    let json_format = command.format.is_json();
    let command_event = if json_format {
        EVENT_DISCORD_COMMAND_SESSION_INJECTION_JSON_REPLIED
    } else {
        EVENT_DISCORD_COMMAND_SESSION_INJECTION_REPLIED
    };
    let response =
        build_session_injection_response(command.action, json_format, agent, session_id).await;
    send_response(channel, &msg.recipient, response, msg, command_event).await;
}

async fn build_session_injection_response(
    action: SessionInjectionAction,
    json_format: bool,
    agent: &Arc<Agent>,
    session_id: &str,
) -> String {
    match action {
        SessionInjectionAction::Status => {
            build_session_injection_status_response(agent, session_id, json_format).await
        }
        SessionInjectionAction::Clear => {
            build_session_injection_clear_response(agent, session_id, json_format).await
        }
        SessionInjectionAction::SetXml(payload) => {
            build_session_injection_set_xml_response(agent, session_id, &payload, json_format).await
        }
    }
}

async fn build_session_injection_status_response(
    agent: &Arc<Agent>,
    session_id: &str,
    json_format: bool,
) -> String {
    match agent.inspect_session_system_prompt_injection(session_id).await {
        Some(snapshot) if json_format => json!({
            "kind": "session_injection",
            "configured": true,
            "qa_count": snapshot.qa_count,
            "updated_at_unix_ms": snapshot.updated_at_unix_ms,
            "xml": snapshot.xml,
        })
        .to_string(),
        Some(snapshot) => format!(
            "Session system prompt injection is configured.\nqa_count={}\nupdated_at_unix_ms={}\nxml_preview:\n{}",
            snapshot.qa_count,
            snapshot.updated_at_unix_ms,
            truncate_preview(&snapshot.xml, 800)
        ),
        None if json_format => json!({
            "kind": "session_injection",
            "configured": false,
            "message": "No system prompt injection is configured for this session.",
        })
        .to_string(),
        None => "No system prompt injection is configured for this session.\nUse `/session inject <qa>...</qa>` to configure it.".to_string(),
    }
}

async fn build_session_injection_clear_response(
    agent: &Arc<Agent>,
    session_id: &str,
    json_format: bool,
) -> String {
    match agent
        .clear_session_system_prompt_injection(session_id)
        .await
    {
        Ok(cleared) if json_format => json!({
            "kind": "session_injection",
            "cleared": cleared,
        })
        .to_string(),
        Ok(true) => "Session system prompt injection cleared.".to_string(),
        Ok(false) => "No session system prompt injection existed to clear.".to_string(),
        Err(error) if json_format => {
            format_command_error_json("session_injection_clear", &error.to_string())
        }
        Err(error) => format!("Failed to clear session system prompt injection: {error}"),
    }
}

async fn build_session_injection_set_xml_response(
    agent: &Arc<Agent>,
    session_id: &str,
    payload: &str,
    json_format: bool,
) -> String {
    match agent
        .upsert_session_system_prompt_injection_xml(session_id, payload)
        .await
    {
        Ok(snapshot) if json_format => json!({
            "kind": "session_injection",
            "configured": true,
            "qa_count": snapshot.qa_count,
            "updated_at_unix_ms": snapshot.updated_at_unix_ms,
        })
        .to_string(),
        Ok(snapshot) => format!(
            "Session system prompt injection updated.\nqa_count={}\nupdated_at_unix_ms={}",
            snapshot.qa_count, snapshot.updated_at_unix_ms
        ),
        Err(error) if json_format => {
            format_command_error_json("session_injection_set", &error.to_string())
        }
        Err(error) => format!("Invalid system prompt injection payload: {error}"),
    }
}

fn truncate_preview(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    if max_chars <= 3 {
        return ".".repeat(max_chars);
    }
    let mut out = String::new();
    for ch in value.chars().take(max_chars.saturating_sub(3)) {
        out.push(ch);
    }
    out.push_str("...");
    out
}
