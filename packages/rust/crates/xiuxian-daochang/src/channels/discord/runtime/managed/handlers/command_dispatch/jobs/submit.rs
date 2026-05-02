use std::sync::Arc;

use crate::channels::managed_commands::SLASH_SCOPE_BACKGROUND_SUBMIT;
use crate::channels::traits::{Channel, ChannelMessage};
use crate::jobs::JobManager;

use crate::channels::discord::runtime::managed::handlers::auth::ensure_slash_command_authorized;
use crate::channels::discord::runtime::managed::handlers::events::{
    EVENT_DISCORD_COMMAND_BACKGROUND_SUBMIT_FAILED_REPLIED,
    EVENT_DISCORD_COMMAND_BACKGROUND_SUBMIT_REPLIED,
};
use crate::channels::discord::runtime::managed::handlers::send::send_response;

pub(in super::super) async fn handle_background_submit(
    channel: &Arc<dyn Channel>,
    msg: &ChannelMessage,
    job_manager: &Arc<JobManager>,
    session_id: &str,
    prompt: String,
) {
    if !ensure_slash_command_authorized(channel, msg, SLASH_SCOPE_BACKGROUND_SUBMIT, "/bg").await {
        return;
    }
    let response = match job_manager
        .submit(session_id, msg.recipient.clone(), prompt)
        .await
    {
        Ok(job_id) => format!(
            "Queued background job `{job_id}`.\nUse `/job {job_id}` for status, `/jobs` for queue health."
        ),
        Err(error) => format!("Failed to queue background job: {error}"),
    };
    let event = if response.starts_with("Queued background job") {
        EVENT_DISCORD_COMMAND_BACKGROUND_SUBMIT_REPLIED
    } else {
        EVENT_DISCORD_COMMAND_BACKGROUND_SUBMIT_FAILED_REPLIED
    };
    send_response(channel, &msg.recipient, response, msg, event).await;
}
