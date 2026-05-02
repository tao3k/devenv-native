use std::sync::PoisonError;

use super::types::ParsedTelegramUpdate;
use crate::channels::telegram::TelegramChannel;
use crate::channels::telegram::channel::identity::{
    normalize_group_identity, normalize_user_identity,
};

pub(super) fn is_user_allowed(channel: &TelegramChannel, identity: &str) -> bool {
    let normalized = normalize_user_identity(identity);
    channel
        .allowed_users
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .any(|user| user == "*" || user == &normalized)
}

pub(super) fn is_identity_in_allowlist(identity: &str, allowlist: &[String]) -> bool {
    let normalized = normalize_user_identity(identity);
    if normalized.is_empty() {
        return false;
    }
    allowlist
        .iter()
        .any(|entry| entry == "*" || entry == &normalized)
}

pub(super) fn is_group_allowed(channel: &TelegramChannel, chat_id: &str) -> bool {
    let normalized = normalize_group_identity(chat_id);
    channel
        .allowed_groups
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .any(|group| group == "*" || group == &normalized)
}

pub(super) fn resolve_sender_acl(
    channel: &TelegramChannel,
    parsed: &ParsedTelegramUpdate<'_>,
) -> (bool, bool) {
    let allowed_by_group =
        parsed.chat_id.starts_with('-') && is_group_allowed(channel, &parsed.chat_id);
    let allowed_by_user = parsed
        .user_id
        .as_deref()
        .is_some_and(|identity| is_user_allowed(channel, identity));
    (allowed_by_group, allowed_by_user)
}

pub(super) fn log_unauthorized_sender(parsed: &ParsedTelegramUpdate<'_>) {
    tracing::warn!(
        "Telegram: ignoring message from unauthorized user. \
         Add to allowed_users (user_id={}, username={}) or allowed_groups (chat_id={}, chat_title={}, chat_type={})",
        parsed.user_id.as_deref().unwrap_or("-"),
        parsed.username.unwrap_or("(not set)"),
        parsed.chat_id,
        parsed.chat_title,
        parsed.chat_type
    );
}
