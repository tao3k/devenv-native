use std::borrow::Cow;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::observability::SessionEvent;

use super::RedisSessionBackend;
use crate::session::{ChatMessage, FunctionCall, ToolCallOut};

const SESSION_CONTEXT_BACKUP_META_PREFIX: &str = "__session_context_backup_meta__:";
const COMPACT_CHAT_MESSAGE_VERSION: u8 = 1;

#[derive(Debug, Deserialize)]
struct LegacySessionContextBackupMetadataPayload {
    messages: usize,
    summary_segments: usize,
    saved_at_unix_ms: u64,
}

#[derive(Debug, Serialize)]
struct CompactChatMessagePayload<'a> {
    #[serde(rename = "v")]
    version: u8,
    #[serde(rename = "r")]
    role: &'a str,
    #[serde(rename = "c", skip_serializing_if = "Option::is_none")]
    content: Option<Cow<'a, str>>,
    #[serde(rename = "tc", skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<CompactToolCallPayload<'a>>>,
    #[serde(rename = "ti", skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<&'a str>,
    #[serde(rename = "n", skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct CompactToolCallPayload<'a> {
    #[serde(rename = "i")]
    id: &'a str,
    #[serde(rename = "t")]
    typ: &'a str,
    #[serde(rename = "f")]
    function: CompactFunctionCallPayload<'a>,
}

#[derive(Debug, Serialize)]
struct CompactFunctionCallPayload<'a> {
    #[serde(rename = "n")]
    name: &'a str,
    #[serde(rename = "a")]
    arguments: &'a str,
}

#[derive(Debug, Deserialize)]
struct CompactChatMessagePayloadOwned {
    #[serde(rename = "v", default)]
    version: Option<u8>,
    #[serde(rename = "r")]
    role: String,
    #[serde(rename = "c")]
    content: Option<String>,
    #[serde(rename = "tc")]
    tool_calls: Option<Vec<CompactToolCallPayloadOwned>>,
    #[serde(rename = "ti")]
    tool_call_id: Option<String>,
    #[serde(rename = "n")]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CompactToolCallPayloadOwned {
    #[serde(rename = "i")]
    id: String,
    #[serde(rename = "t")]
    typ: String,
    #[serde(rename = "f")]
    function: CompactFunctionCallPayloadOwned,
}

#[derive(Debug, Deserialize)]
struct CompactFunctionCallPayloadOwned {
    #[serde(rename = "n")]
    name: String,
    #[serde(rename = "a")]
    arguments: String,
}

impl From<CompactChatMessagePayloadOwned> for ChatMessage {
    fn from(value: CompactChatMessagePayloadOwned) -> Self {
        let _version = value.version.unwrap_or(COMPACT_CHAT_MESSAGE_VERSION);
        let tool_calls = value.tool_calls.map(|calls| {
            calls
                .into_iter()
                .map(|call| ToolCallOut {
                    id: call.id,
                    typ: call.typ,
                    function: FunctionCall {
                        name: call.function.name,
                        arguments: call.function.arguments,
                    },
                })
                .collect::<Vec<_>>()
        });
        Self {
            role: value.role,
            content: value.content,
            tool_calls,
            tool_call_id: value.tool_call_id,
            name: value.name,
        }
    }
}

#[derive(Debug)]
pub(super) struct EncodedChatMessagePayload {
    pub(super) payload: String,
    pub(super) content_truncated: bool,
}

fn maybe_truncate_content_to_chars(
    content: &str,
    max_chars: Option<usize>,
) -> (Cow<'_, str>, bool) {
    let Some(max_chars) = max_chars.filter(|value| *value > 0) else {
        return (Cow::Borrowed(content), false);
    };
    if content.chars().count() <= max_chars {
        return (Cow::Borrowed(content), false);
    }
    let truncated = content.chars().take(max_chars).collect::<String>();
    (Cow::Owned(truncated), true)
}

pub(super) fn encode_chat_message_payload(
    message: &ChatMessage,
    max_content_chars: Option<usize>,
) -> Result<EncodedChatMessagePayload> {
    let mut content_truncated = false;
    let content = message.content.as_deref().map(|text| {
        let (normalized, truncated) = maybe_truncate_content_to_chars(text, max_content_chars);
        content_truncated = truncated;
        normalized
    });
    let tool_calls = message.tool_calls.as_ref().map(|calls| {
        calls
            .iter()
            .map(|call| CompactToolCallPayload {
                id: call.id.as_str(),
                typ: call.typ.as_str(),
                function: CompactFunctionCallPayload {
                    name: call.function.name.as_str(),
                    arguments: call.function.arguments.as_str(),
                },
            })
            .collect::<Vec<_>>()
    });
    let compact_payload = CompactChatMessagePayload {
        version: COMPACT_CHAT_MESSAGE_VERSION,
        role: message.role.as_str(),
        content,
        tool_calls,
        tool_call_id: message.tool_call_id.as_deref(),
        name: message.name.as_deref(),
    };
    let payload = serde_json::to_string(&compact_payload)
        .context("failed to serialize compact chat message")?;
    Ok(EncodedChatMessagePayload {
        payload,
        content_truncated,
    })
}

pub(crate) fn test_encode_chat_message_payload(
    message: &ChatMessage,
    max_content_chars: Option<usize>,
) -> Result<(String, bool)> {
    let encoded = encode_chat_message_payload(message, max_content_chars)?;
    Ok((encoded.payload, encoded.content_truncated))
}

impl RedisSessionBackend {
    pub(crate) async fn append_messages(
        &self,
        session_id: &str,
        messages: &[ChatMessage],
    ) -> Result<()> {
        if messages.is_empty() {
            return Ok(());
        }
        let key = self.messages_key(session_id);
        let mut encoded = Vec::with_capacity(messages.len());
        let mut content_truncated_messages = 0usize;
        for message in messages {
            let encoded_message =
                encode_chat_message_payload(message, self.message_content_max_chars)
                    .context("failed to encode chat messages for redis")?;
            if encoded_message.content_truncated {
                content_truncated_messages = content_truncated_messages.saturating_add(1);
            }
            encoded.push(encoded_message.payload);
        }
        let ttl_secs = self.ttl_secs;

        self.run_pipeline::<(), _>("append_messages", || {
            let mut pipe = redis::pipe();
            pipe.atomic();
            pipe.cmd("RPUSH").arg(&key);
            for payload in &encoded {
                pipe.arg(payload);
            }
            pipe.ignore();
            if let Some(ttl) = ttl_secs {
                pipe.cmd("EXPIRE").arg(&key).arg(ttl).ignore();
            }
            pipe
        })
        .await?;
        tracing::debug!(
            event = SessionEvent::SessionMessagesAppended.as_str(),
            session_id,
            appended_messages = encoded.len(),
            content_truncated_messages,
            message_content_max_chars = ?self.message_content_max_chars,
            ttl_secs = ?ttl_secs,
            "valkey session messages appended"
        );
        Ok(())
    }

    pub(crate) async fn replace_messages(
        &self,
        session_id: &str,
        messages: &[ChatMessage],
    ) -> Result<usize> {
        let key = self.messages_key(session_id);
        let mut encoded = Vec::with_capacity(messages.len());
        let mut content_truncated_messages = 0usize;
        for message in messages {
            let encoded_message =
                encode_chat_message_payload(message, self.message_content_max_chars)
                    .context("failed to encode chat messages for redis replace")?;
            if encoded_message.content_truncated {
                content_truncated_messages = content_truncated_messages.saturating_add(1);
            }
            encoded.push(encoded_message.payload);
        }
        let ttl_secs = self.ttl_secs.unwrap_or(0);
        let message_count = encoded.len();
        let message_count_i64 = i64::try_from(message_count)
            .context("message count overflow while replacing redis session messages")?;
        let script = r#"
local key = KEYS[1]
local ttl = tonumber(ARGV[1]) or 0
local count = tonumber(ARGV[2]) or 0
redis.call("DEL", key)
if count > 0 then
  for i = 1, count do
    redis.call("RPUSH", key, ARGV[2 + i])
  end
  if ttl > 0 then
    redis.call("EXPIRE", key, ttl)
  end
end
return count
"#;

        let replaced_count = self
            .run_command::<usize, _>("replace_messages", || {
                let mut cmd = redis::cmd("EVAL");
                cmd.arg(script)
                    .arg(1)
                    .arg(&key)
                    .arg(ttl_secs)
                    .arg(message_count_i64);
                for payload in &encoded {
                    cmd.arg(payload);
                }
                cmd
            })
            .await?;
        tracing::debug!(
            event = SessionEvent::SessionMessagesReplaced.as_str(),
            session_id,
            replaced_messages = replaced_count,
            content_truncated_messages,
            message_content_max_chars = ?self.message_content_max_chars,
            ttl_secs = ?self.ttl_secs,
            "valkey session messages replaced atomically"
        );
        Ok(replaced_count)
    }

    pub(crate) async fn get_messages(&self, session_id: &str) -> Result<Vec<ChatMessage>> {
        let key = self.messages_key(session_id);
        let payloads = self
            .run_command::<Vec<String>, _>("get_messages", || {
                let mut cmd = redis::cmd("LRANGE");
                cmd.arg(&key).arg(0).arg(-1);
                cmd
            })
            .await?;
        let mut out = Vec::with_capacity(payloads.len());
        let mut invalid_payloads = 0usize;
        for payload in payloads {
            match decode_chat_message_payload(session_id, &payload) {
                Ok(message) => out.push(message),
                Err(error) => {
                    invalid_payloads += 1;
                    tracing::warn!(
                        event = SessionEvent::SessionMessagesLoaded.as_str(),
                        session_id,
                        error = %error,
                        "invalid chat message payload in redis session store"
                    );
                }
            }
        }
        tracing::debug!(
            event = SessionEvent::SessionMessagesLoaded.as_str(),
            session_id,
            loaded_messages = out.len(),
            invalid_payloads,
            "valkey session messages loaded"
        );
        Ok(out)
    }

    pub(crate) async fn clear_messages(&self, session_id: &str) -> Result<()> {
        let key = self.messages_key(session_id);
        let _ = self
            .run_command::<i64, _>("clear_messages", || {
                let mut cmd = redis::cmd("DEL");
                cmd.arg(&key);
                cmd
            })
            .await?;
        tracing::debug!(
            event = SessionEvent::SessionMessagesCleared.as_str(),
            session_id,
            "valkey session messages cleared"
        );
        Ok(())
    }
}

pub(super) fn decode_chat_message_payload(
    session_id: &str,
    payload: &str,
) -> std::result::Result<ChatMessage, serde_json::Error> {
    match serde_json::from_str::<ChatMessage>(payload) {
        Ok(message) => Ok(message),
        Err(chat_message_error) => {
            if let Ok(compact) = serde_json::from_str::<CompactChatMessagePayloadOwned>(payload) {
                return Ok(compact.into());
            }
            if session_id.starts_with(SESSION_CONTEXT_BACKUP_META_PREFIX) {
                let metadata =
                    serde_json::from_str::<LegacySessionContextBackupMetadataPayload>(payload);
                if let Ok(metadata) = metadata {
                    tracing::debug!(
                        event = SessionEvent::ContextBackupCaptured.as_str(),
                        session_id,
                        legacy_messages = metadata.messages,
                        legacy_summary_segments = metadata.summary_segments,
                        legacy_saved_at_unix_ms = metadata.saved_at_unix_ms,
                        "decoded legacy session context backup metadata payload"
                    );
                    return Ok(ChatMessage {
                        role: "system".to_string(),
                        content: Some(payload.to_string()),
                        tool_calls: None,
                        tool_call_id: None,
                        name: None,
                    });
                }
            }
            Err(chat_message_error)
        }
    }
}

pub(crate) fn test_decode_chat_message_payload(
    session_id: &str,
    payload: &str,
) -> std::result::Result<ChatMessage, serde_json::Error> {
    decode_chat_message_payload(session_id, payload)
}
