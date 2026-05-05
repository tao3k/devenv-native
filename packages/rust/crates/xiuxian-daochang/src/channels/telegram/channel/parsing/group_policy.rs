use std::sync::PoisonError;

use super::acl::is_identity_in_allowlist;
use super::types::ParsedTelegramUpdate;
use crate::channels::telegram::TelegramChannel;
use crate::channels::telegram::channel::group_policy::TelegramGroupPolicyMode;

pub(super) fn group_policy_allows_message(
    channel: &TelegramChannel,
    parsed: &ParsedTelegramUpdate<'_>,
    allowed_by_user: bool,
    user_identity: &str,
) -> bool {
    if !parsed.chat_id.starts_with('-') {
        return true;
    }

    let effective_policy = channel
        .group_policy_config
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .resolve(&parsed.chat_id, parsed.message_thread_id);

    if !effective_policy.enabled
        || matches!(
            effective_policy.group_policy,
            TelegramGroupPolicyMode::Disabled
        )
    {
        tracing::debug!(
            chat_id = %parsed.chat_id,
            user_id = %user_identity,
            message_thread_id = ?parsed.message_thread_id,
            "telegram group message ignored: group policy disabled"
        );
        return false;
    }

    if matches!(
        effective_policy.group_policy,
        TelegramGroupPolicyMode::Allowlist
    ) {
        let sender_allowed = match &effective_policy.allow_from {
            Some(entries) => is_identity_in_allowlist(user_identity, entries),
            None => allowed_by_user,
        };
        if !sender_allowed {
            tracing::debug!(
                chat_id = %parsed.chat_id,
                user_id = %user_identity,
                message_thread_id = ?parsed.message_thread_id,
                "telegram group message ignored: sender not in allowlist policy"
            );
            return false;
        }
    }

    if effective_policy.require_mention
        && !is_message_triggered_for_group(parsed.message, parsed.text)
    {
        tracing::debug!(
            chat_id = %parsed.chat_id,
            user_id = %user_identity,
            message_thread_id = ?parsed.message_thread_id,
            "telegram group message ignored: require_mention enabled and no mention trigger detected"
        );
        return false;
    }

    true
}

fn is_message_triggered_for_group(message: &serde_json::Value, text: &str) -> bool {
    let trimmed = text.trim_start();
    if trimmed.starts_with('/') {
        return true;
    }
    if trimmed.contains('@') {
        return true;
    }
    if message
        .get("reply_to_message")
        .and_then(|reply| reply.get("from"))
        .and_then(|from| from.get("is_bot"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return true;
    }
    message
        .get("entities")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|entities| {
            entities.iter().any(|entity| {
                entity
                    .get("type")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|entity_type| {
                        matches!(entity_type, "mention" | "text_mention" | "bot_command")
                    })
            })
        })
}
