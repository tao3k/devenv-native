use crate::channels::telegram::channel::TelegramChannel;
use crate::channels::telegram::session_partition::TelegramMessageThreadId;
use crate::channels::traits::ChannelMessage;

use super::types::ParsedTelegramUpdate;

pub(super) fn extract_update_message(
    update: &serde_json::Value,
) -> Option<ParsedTelegramUpdate<'_>> {
    let message = update.get("message")?;
    let text = message
        .get("text")
        .and_then(serde_json::Value::as_str)
        .or_else(|| message.get("caption").and_then(serde_json::Value::as_str))
        .or_else(|| {
            message
                .get("photo")
                .and_then(serde_json::Value::as_array)
                .filter(|photos| !photos.is_empty())
                .map(|_| "[telegram-photo]")
        })
        .or_else(|| {
            message
                .get("document")
                .and_then(|document| document.get("mime_type"))
                .and_then(serde_json::Value::as_str)
                .filter(|mime| mime.starts_with("image/"))
                .map(|_| "[telegram-image]")
        })?;
    let chat = message.get("chat")?;
    let chat_id = chat
        .get("id")
        .and_then(serde_json::Value::as_i64)
        .map(|id| id.to_string())?;
    let chat_title = chat
        .get("title")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("(not set)");
    let chat_type = chat
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("(not set)");
    let username = message
        .get("from")
        .and_then(|from| from.get("username"))
        .and_then(serde_json::Value::as_str);
    let user_id = message
        .get("from")
        .and_then(|from| from.get("id"))
        .and_then(serde_json::Value::as_i64)
        .map(|id| id.to_string());
    let message_thread_id = message
        .get("message_thread_id")
        .and_then(serde_json::Value::as_i64);
    let message_id = message
        .get("message_id")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or_default();
    let update_id = update
        .get("update_id")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or_default();

    Some(ParsedTelegramUpdate {
        message,
        text,
        chat_id,
        chat_title,
        chat_type,
        username,
        user_id,
        message_thread_id,
        message_id,
        update_id,
    })
}

fn build_session_key(
    channel: &TelegramChannel,
    chat_id: &str,
    user_identity: &str,
    message_thread_id: Option<i64>,
) -> String {
    channel.session_partition().build_session_key(
        chat_id,
        user_identity,
        message_thread_id.map(TelegramMessageThreadId::new),
    )
}

pub(super) fn build_channel_message_from_parsed(
    channel: &TelegramChannel,
    parsed: &ParsedTelegramUpdate<'_>,
    user_identity: &str,
) -> ChannelMessage {
    let session_key = build_session_key(
        channel,
        &parsed.chat_id,
        user_identity,
        parsed.message_thread_id,
    );
    ChannelMessage {
        id: format!(
            "telegram_{}_{}_{}",
            parsed.chat_id, parsed.message_id, parsed.update_id
        ),
        sender: user_identity.to_string(),
        recipient: parsed.recipient(),
        session_key,
        content: parsed.text.to_string(),
        attachments: Vec::new(),
        channel: "telegram".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    }
}
