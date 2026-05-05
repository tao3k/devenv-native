use std::sync::Arc;

use crate::channels::traits::{Channel, ChannelMessage};

use super::events::{
    EVENT_DISCORD_COMMAND_CONTROL_ADMIN_REQUIRED_REPLIED,
    EVENT_DISCORD_COMMAND_SLASH_PERMISSION_REQUIRED_REPLIED,
};
use super::send::send_response;
use crate::channels::discord::runtime::managed::replies::{
    format_control_command_admin_required, format_slash_command_permission_required,
};

pub(super) async fn ensure_control_command_authorized(
    channel: &Arc<dyn Channel>,
    msg: &ChannelMessage,
    command: &str,
) -> bool {
    if channel.is_authorized_for_control_command_for_recipient(
        &msg.sender,
        &msg.content,
        &msg.recipient,
    ) {
        return true;
    }
    let response = format_control_command_admin_required(command, &msg.sender);
    send_response(
        channel,
        &msg.recipient,
        response,
        msg,
        EVENT_DISCORD_COMMAND_CONTROL_ADMIN_REQUIRED_REPLIED,
    )
    .await;
    false
}

pub(super) async fn ensure_slash_command_authorized(
    channel: &Arc<dyn Channel>,
    msg: &ChannelMessage,
    scope: &str,
    command_label: &str,
) -> bool {
    if channel.is_authorized_for_slash_command_for_recipient(&msg.sender, scope, &msg.recipient) {
        return true;
    }
    let response = format_slash_command_permission_required(command_label, &msg.sender);
    send_response(
        channel,
        &msg.recipient,
        response,
        msg,
        EVENT_DISCORD_COMMAND_SLASH_PERMISSION_REQUIRED_REPLIED,
    )
    .await;
    false
}
