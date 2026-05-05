use std::sync::Arc;

use crate::channels::managed_commands::SLASH_SCOPE_BACKGROUND_SUBMIT as TELEGRAM_SLASH_SCOPE_BACKGROUND_SUBMIT;
use crate::channels::managed_runtime::turn::compose_turn_content;
use crate::channels::telegram::commands::parse_background_prompt;
use crate::channels::traits::{Channel, ChannelMessage};
use crate::jobs::JobManager;

use super::{
    EVENT_TELEGRAM_COMMAND_BACKGROUND_SUBMIT_FAILED_REPLIED,
    EVENT_TELEGRAM_COMMAND_BACKGROUND_SUBMIT_REPLIED,
};
use crate::channels::telegram::runtime::jobs::command_handlers::slash_acl::ensure_slash_command_authorized;
use crate::channels::telegram::runtime::jobs::observability::send_with_observability;

pub(in crate::channels::telegram::runtime::jobs) async fn try_handle_background_prompt_command(
    msg: &ChannelMessage,
    channel: &Arc<dyn Channel>,
    job_manager: &Arc<JobManager>,
    session_id: &str,
) -> bool {
    let Some((prompt, prompt_origin)) = resolve_background_prompt(msg) else {
        return false;
    };

    if matches!(prompt_origin, BackgroundPromptOrigin::ExplicitCommand)
        && !ensure_slash_command_authorized(
            channel,
            msg,
            TELEGRAM_SLASH_SCOPE_BACKGROUND_SUBMIT,
            "/bg",
        )
        .await
    {
        return true;
    }

    match job_manager
        .submit(session_id, msg.recipient.clone(), prompt)
        .await
    {
        Ok(job_id) => {
            let mut ack = format!(
                "Queued background job `{job_id}`.\nUse `/job {job_id}` for status, `/jobs` for queue health."
            );
            if matches!(prompt_origin, BackgroundPromptOrigin::AutoImageAttachment) {
                ack.push_str("\nAuto-routed image message to background execution.");
            }
            send_with_observability(
                channel,
                &ack,
                &msg.recipient,
                "Failed to send background ack",
                Some(EVENT_TELEGRAM_COMMAND_BACKGROUND_SUBMIT_REPLIED),
                Some(&msg.session_key),
            )
            .await;
        }
        Err(error) => {
            let failure = format!("Failed to queue background job: {error}");
            send_with_observability(
                channel,
                &failure,
                &msg.recipient,
                "Failed to send background queue failure",
                Some(EVENT_TELEGRAM_COMMAND_BACKGROUND_SUBMIT_FAILED_REPLIED),
                Some(&msg.session_key),
            )
            .await;
        }
    }
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackgroundPromptOrigin {
    ExplicitCommand,
    AutoImageAttachment,
}

fn resolve_background_prompt(msg: &ChannelMessage) -> Option<(String, BackgroundPromptOrigin)> {
    if let Some(prompt) = parse_background_prompt(&msg.content) {
        let prompt = compose_prompt_with_attachments(msg, prompt);
        return Some((prompt, BackgroundPromptOrigin::ExplicitCommand));
    }

    if should_auto_route_image_message(msg) {
        return Some((
            compose_turn_content(msg),
            BackgroundPromptOrigin::AutoImageAttachment,
        ));
    }

    None
}

fn should_auto_route_image_message(msg: &ChannelMessage) -> bool {
    if msg.attachments.is_empty() {
        return false;
    }
    !msg.content.trim_start().starts_with('/')
}

fn compose_prompt_with_attachments(msg: &ChannelMessage, prompt: String) -> String {
    if msg.attachments.is_empty() {
        return prompt;
    }
    let mut synthetic = msg.clone();
    synthetic.content = prompt;
    compose_turn_content(&synthetic)
}
