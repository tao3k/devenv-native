use std::sync::Arc;

use crate::agent::Agent;
use crate::channels::traits::{Channel, ChannelMessage};

use crate::channels::discord::runtime::managed::handlers::auth::ensure_control_command_authorized;
use crate::channels::discord::runtime::managed::handlers::events::{
    EVENT_DISCORD_COMMAND_SESSION_RESUME_DROP_REPLIED,
    EVENT_DISCORD_COMMAND_SESSION_RESUME_REPLIED,
    EVENT_DISCORD_COMMAND_SESSION_RESUME_STATUS_REPLIED,
};
use crate::channels::discord::runtime::managed::handlers::send::send_response;
use crate::channels::discord::runtime::managed::parsing::ResumeCommand;

pub(in super::super) async fn handle_resume(
    agent: &Arc<Agent>,
    channel: &Arc<dyn Channel>,
    msg: &ChannelMessage,
    session_id: &str,
    resume_command: ResumeCommand,
) {
    let resume_requires_admin =
        matches!(resume_command, ResumeCommand::Restore | ResumeCommand::Drop);
    if resume_requires_admin {
        let command = match resume_command {
            ResumeCommand::Restore => "/resume",
            ResumeCommand::Status => "/resume status",
            ResumeCommand::Drop => "/resume drop",
        };
        if !ensure_control_command_authorized(channel, msg, command).await {
            return;
        }
    }

    let (event, response) = match resume_command {
        ResumeCommand::Restore => (
            EVENT_DISCORD_COMMAND_SESSION_RESUME_REPLIED,
            match agent.resume_context_window(session_id).await {
                Ok(Some(stats)) => format!(
                    "Session context restored.\nmessages_restored={} summary_segments_restored={}",
                    stats.messages, stats.summary_segments
                ),
                Ok(None) => {
                    "No saved session context snapshot found. Use `/reset` or `/clear` first."
                        .to_string()
                }
                Err(error) => format!("Failed to restore session context: {error}"),
            },
        ),
        ResumeCommand::Status => (
            EVENT_DISCORD_COMMAND_SESSION_RESUME_STATUS_REPLIED,
            match agent.peek_context_window_backup(session_id).await {
                Ok(Some(info)) => {
                    let mut lines = vec![
                        "Saved session context snapshot:".to_string(),
                        format!("messages={}", info.messages),
                        format!("summary_segments={}", info.summary_segments),
                    ];
                    if let Some(saved_at_unix_ms) = info.saved_at_unix_ms {
                        lines.push(format!("saved_at_unix_ms={saved_at_unix_ms}"));
                    }
                    if let Some(saved_age_secs) = info.saved_age_secs {
                        lines.push(format!("saved_age_secs={saved_age_secs}"));
                    }
                    lines.push("Use `/resume` to restore.".to_string());
                    lines.join("\n")
                }
                Ok(None) => "No saved session context snapshot found.".to_string(),
                Err(error) => format!("Failed to inspect session context snapshot: {error}"),
            },
        ),
        ResumeCommand::Drop => (
            EVENT_DISCORD_COMMAND_SESSION_RESUME_DROP_REPLIED,
            match agent.drop_context_window_backup(session_id).await {
                Ok(true) => "Saved session context snapshot dropped.".to_string(),
                Ok(false) => "No saved session context snapshot found to drop.".to_string(),
                Err(error) => format!("Failed to drop saved session context snapshot: {error}"),
            },
        ),
    };
    send_response(channel, &msg.recipient, response, msg, event).await;
}
