//! Telegram update parsing entrypoint.

use crate::channels::ChannelMessage;
use crate::channels::telegram::channel::TelegramChannel;

use super::acl::{log_unauthorized_sender, resolve_sender_acl};
use super::group_policy::group_policy_allows_message;
use super::update_message::{build_channel_message_from_parsed, extract_update_message};

impl TelegramChannel {
    /// Parse a Telegram update into a channel message (returns None for unsupported updates).
    pub fn parse_update_message(&self, update: &serde_json::Value) -> Option<ChannelMessage> {
        self.ensure_acl_fresh();
        let parsed = extract_update_message(update)?;
        let (allowed_by_group, allowed_by_user) = resolve_sender_acl(self, &parsed);
        if !allowed_by_group && !allowed_by_user {
            log_unauthorized_sender(&parsed);
            return None;
        }

        let user_identity = parsed.user_identity();
        if !group_policy_allows_message(self, &parsed, allowed_by_user, &user_identity) {
            return None;
        }

        Some(build_channel_message_from_parsed(
            self,
            &parsed,
            &user_identity,
        ))
    }
}
