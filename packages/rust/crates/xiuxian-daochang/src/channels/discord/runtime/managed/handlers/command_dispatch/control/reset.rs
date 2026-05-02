use std::sync::Arc;

use crate::agent::Agent;
use crate::channels::traits::{Channel, ChannelMessage};

use crate::channels::discord::runtime::managed::handlers::auth::ensure_control_command_authorized;
use crate::channels::discord::runtime::managed::handlers::events::EVENT_DISCORD_COMMAND_SESSION_RESET_REPLIED;
use crate::channels::discord::runtime::managed::handlers::send::send_response;

pub(in super::super) async fn handle_reset(
    agent: &Arc<Agent>,
    channel: &Arc<dyn Channel>,
    msg: &ChannelMessage,
    session_id: &str,
) {
    if !ensure_control_command_authorized(channel, msg, "/reset").await {
        return;
    }
    let response = match agent.reset_context_window(session_id).await {
        Ok(stats) => {
            if stats.messages > 0 || stats.summary_segments > 0 {
                format!(
                    "Session context reset.\nmessages_cleared={} summary_segments_cleared={}\nUse `/resume` to restore this session context.\nLong-term memory and knowledge stores are unchanged.",
                    stats.messages, stats.summary_segments
                )
            } else {
                "Session context reset.\nmessages_cleared=0 summary_segments_cleared=0\nNo active context snapshot was created because this session is already empty.\nLong-term memory and knowledge stores are unchanged."
                    .to_string()
            }
        }
        Err(error) => format!("Failed to reset session context: {error}"),
    };
    send_response(
        channel,
        &msg.recipient,
        response,
        msg,
        EVENT_DISCORD_COMMAND_SESSION_RESET_REPLIED,
    )
    .await;
}
