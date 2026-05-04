//! Redis message-payload helpers exposed for integration tests.

use crate::session::{ChatMessage, redis_backend::message_store};

use super::TestSupportResult;

/// Encoded compact chat-message payload metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedChatMessagePayload {
    pub payload: String,
    pub content_truncated: bool,
}

/// Encode one chat message to compact redis payload format.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn encode_chat_message_payload(
    message: &ChatMessage,
    max_content_chars: Option<usize>,
) -> TestSupportResult<EncodedChatMessagePayload> {
    let (payload, content_truncated) =
        message_store::test_encode_chat_message_payload(message, max_content_chars)?;
    Ok(EncodedChatMessagePayload {
        payload,
        content_truncated,
    })
}

/// Decode one redis payload into chat-message shape.
///
/// # Errors
///
/// Returns an error when payload JSON cannot be parsed in any supported schema.
pub fn decode_chat_message_payload(
    session_id: impl AsRef<str>,
    payload: &str,
) -> Result<ChatMessage, serde_json::Error> {
    message_store::test_decode_chat_message_payload(session_id.as_ref(), payload)
}
