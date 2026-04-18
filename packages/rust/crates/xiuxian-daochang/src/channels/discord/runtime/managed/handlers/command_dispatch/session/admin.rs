use std::sync::Arc;

use serde_json::json;

use crate::channels::discord::runtime::managed::handlers::auth::ensure_control_command_authorized;
use crate::channels::discord::runtime::managed::handlers::events::{
    EVENT_DISCORD_COMMAND_SESSION_ADMIN_JSON_REPLIED, EVENT_DISCORD_COMMAND_SESSION_ADMIN_REPLIED,
};
use crate::channels::discord::runtime::managed::handlers::send::send_response;
use crate::channels::discord::runtime::managed::replies::format_command_error_json;
use crate::channels::telegram::commands::{SessionAdminAction, SessionAdminCommand};
use crate::channels::traits::{Channel, ChannelMessage, RecipientCommandAdminUsersMutation};

pub(in super::super) async fn handle_session_admin(
    channel: &Arc<dyn Channel>,
    msg: &ChannelMessage,
    command: SessionAdminCommand,
) {
    if !ensure_control_command_authorized(channel, msg, "/session admin").await {
        return;
    }

    let command_event = if command.format.is_json() {
        EVENT_DISCORD_COMMAND_SESSION_ADMIN_JSON_REPLIED
    } else {
        EVENT_DISCORD_COMMAND_SESSION_ADMIN_REPLIED
    };
    let response = match command.action {
        SessionAdminAction::List => match channel.recipient_command_admin_users(&msg.recipient) {
            Ok(admin_users) if command.format.is_json() => {
                format_session_admin_status_json(&msg.recipient, admin_users.as_deref())
            }
            Ok(admin_users) => format_session_admin_status(&msg.recipient, admin_users.as_deref()),
            Err(error) if command.format.is_json() => {
                format_command_error_json("session_admin_status", &error.to_string())
            }
            Err(error) => format!("Failed to inspect session delegated admins: {error}"),
        },
        SessionAdminAction::Set(entries) => update_session_admin_users(
            channel,
            &msg.recipient,
            RecipientCommandAdminUsersMutation::Set(entries),
            "set",
            command.format.is_json(),
        ),
        SessionAdminAction::Add(entries) => update_session_admin_users(
            channel,
            &msg.recipient,
            RecipientCommandAdminUsersMutation::Add(entries),
            "add",
            command.format.is_json(),
        ),
        SessionAdminAction::Remove(entries) => update_session_admin_users(
            channel,
            &msg.recipient,
            RecipientCommandAdminUsersMutation::Remove(entries),
            "remove",
            command.format.is_json(),
        ),
        SessionAdminAction::Clear => update_session_admin_users(
            channel,
            &msg.recipient,
            RecipientCommandAdminUsersMutation::Clear,
            "clear",
            command.format.is_json(),
        ),
    };

    send_response(channel, &msg.recipient, response, msg, command_event).await;
}

fn update_session_admin_users(
    channel: &Arc<dyn Channel>,
    recipient: &str,
    mutation: RecipientCommandAdminUsersMutation,
    action: &str,
    json_format: bool,
) -> String {
    match channel.mutate_recipient_command_admin_users(recipient, mutation) {
        Ok(admin_users) if json_format => {
            format_session_admin_updated_json(action, recipient, admin_users.as_deref())
        }
        Ok(admin_users) => format_session_admin_updated(action, recipient, admin_users.as_deref()),
        Err(error) if json_format => {
            format_command_error_json("session_admin_update", &error.to_string())
        }
        Err(error) => format!("Failed to update session delegated admins: {error}"),
    }
}

fn format_session_admin_status(recipient: &str, override_admin_users: Option<&[String]>) -> String {
    let scope = scope_from_recipient(recipient);
    [
        "Session delegated admins.".to_string(),
        format!("recipient={recipient}"),
        format!("scope={scope}"),
        format!(
            "override_admin_users={}",
            render_admin_users_for_dashboard(override_admin_users)
        ),
        "note=override list is used only at admin_users fallback stage; clear returns to inherited ACL.".to_string(),
    ]
    .join("\n")
}

fn format_session_admin_status_json(
    recipient: &str,
    override_admin_users: Option<&[String]>,
) -> String {
    json!({
        "kind": "session_admin",
        "updated": false,
        "recipient": recipient,
        "scope": scope_from_recipient(recipient),
        "override_admin_users": override_admin_users,
        "note": "override list is used only at admin_users fallback stage; clear returns to inherited ACL",
    })
    .to_string()
}

fn format_session_admin_updated(
    action: &str,
    recipient: &str,
    override_admin_users: Option<&[String]>,
) -> String {
    let scope = scope_from_recipient(recipient);
    [
        "Session delegated admins updated.".to_string(),
        format!("action={action}"),
        format!("recipient={recipient}"),
        format!("scope={scope}"),
        format!(
            "override_admin_users={}",
            render_admin_users_for_dashboard(override_admin_users)
        ),
    ]
    .join("\n")
}

fn format_session_admin_updated_json(
    action: &str,
    recipient: &str,
    override_admin_users: Option<&[String]>,
) -> String {
    json!({
        "kind": "session_admin",
        "updated": true,
        "action": action,
        "recipient": recipient,
        "scope": scope_from_recipient(recipient),
        "override_admin_users": override_admin_users,
    })
    .to_string()
}

fn scope_from_recipient(recipient: &str) -> &'static str {
    let (chat_id, has_thread) = match recipient.split_once(':') {
        Some((chat, thread)) if !chat.is_empty() && !thread.is_empty() => (chat, true),
        _ => (recipient, false),
    };
    if !chat_id.starts_with('-') {
        return "direct";
    }
    if has_thread { "topic" } else { "group" }
}

fn render_admin_users_for_dashboard(override_admin_users: Option<&[String]>) -> String {
    match override_admin_users {
        Some([]) | None => "(inherit)".to_string(),
        Some(entries) => entries.join(","),
    }
}
