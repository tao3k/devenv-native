//! Unified streaming event model and wire serialization.

use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use super::traits::StreamingOutcome;

/// Unified event model for all CLI streaming outputs with zero-copy text.
///
/// Every provider must map its native JSON events into this common enum,
/// allowing Qianji nodes to react to intermediate states before the full
/// response is finished.
///
/// # Zero-Copy Guarantee
///
/// All text fields use `Arc<str>` for efficient sharing without heap duplication.
#[derive(Debug, Clone, PartialEq)]
pub enum ZhenfaStreamingEvent {
    /// Chain of thought / reasoning content.
    Thought(Arc<str>),
    /// Direct text output delta.
    TextDelta(Arc<str>),
    /// Agent is requesting a tool invocation.
    ToolCall {
        /// Unique identifier for this tool call.
        id: Arc<str>,
        /// Name of the tool being invoked.
        name: Arc<str>,
        /// Input parameters for the tool.
        input: Value,
    },
    /// Tool execution result.
    ToolResult {
        /// ID of the corresponding tool call.
        id: Arc<str>,
        /// Output from the tool execution.
        output: Value,
    },
    /// System status message (e.g., "Scanning files...").
    Status(Arc<str>),
    /// Progress indicator with percentage (0-100).
    Progress {
        /// Current step description.
        message: Arc<str>,
        /// Progress percentage (0-100).
        percent: u8,
    },
    /// End of stream with final outcome.
    Finished(StreamingOutcome),
    /// Error occurred during streaming.
    Error {
        /// Error code or type.
        code: Arc<str>,
        /// Human-readable error message.
        message: Arc<str>,
    },
}

impl Serialize for ZhenfaStreamingEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        match self {
            Self::Thought(text) => {
                map.serialize_entry("type", "Thought")?;
                map.serialize_entry("text", text.as_ref())?;
            }
            Self::TextDelta(text) => {
                map.serialize_entry("type", "TextDelta")?;
                map.serialize_entry("text", text.as_ref())?;
            }
            Self::ToolCall { id, name, input } => {
                map.serialize_entry("type", "ToolCall")?;
                map.serialize_entry("id", id.as_ref())?;
                map.serialize_entry("name", name.as_ref())?;
                map.serialize_entry("input", input)?;
            }
            Self::ToolResult { id, output } => {
                map.serialize_entry("type", "ToolResult")?;
                map.serialize_entry("id", id.as_ref())?;
                map.serialize_entry("output", output)?;
            }
            Self::Status(text) => {
                map.serialize_entry("type", "Status")?;
                map.serialize_entry("text", text.as_ref())?;
            }
            Self::Progress { message, percent } => {
                map.serialize_entry("type", "Progress")?;
                map.serialize_entry("message", message.as_ref())?;
                map.serialize_entry("percent", percent)?;
            }
            Self::Finished(outcome) => {
                map.serialize_entry("type", "Finished")?;
                map.serialize_entry("outcome", outcome)?;
            }
            Self::Error { code, message } => {
                map.serialize_entry("type", "Error")?;
                map.serialize_entry("code", code.as_ref())?;
                map.serialize_entry("message", message.as_ref())?;
            }
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for ZhenfaStreamingEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;

        struct ZhenfaStreamingEventVisitor;

        impl<'de> Visitor<'de> for ZhenfaStreamingEventVisitor {
            type Value = ZhenfaStreamingEvent;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a ZhenfaStreamingEvent object")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut event_type: Option<String> = None;
                let mut text: Option<Arc<str>> = None;
                let mut id: Option<Arc<str>> = None;
                let mut name: Option<Arc<str>> = None;
                let mut input: Option<Value> = None;
                let mut output: Option<Value> = None;
                let mut percent: Option<u8> = None;
                let mut message: Option<Arc<str>> = None;
                let mut code: Option<Arc<str>> = None;
                let mut outcome: Option<StreamingOutcome> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "type" => event_type = Some(map.next_value()?),
                        "text" => text = Some(Arc::from(map.next_value::<String>()?)),
                        "id" => id = Some(Arc::from(map.next_value::<String>()?)),
                        "name" => name = Some(Arc::from(map.next_value::<String>()?)),
                        "input" => input = Some(map.next_value()?),
                        "output" => output = Some(map.next_value()?),
                        "percent" => percent = Some(map.next_value()?),
                        "message" => message = Some(Arc::from(map.next_value::<String>()?)),
                        "code" => code = Some(Arc::from(map.next_value::<String>()?)),
                        "outcome" => outcome = Some(map.next_value()?),
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>();
                        }
                    }
                }

                let event_type =
                    event_type.ok_or_else(|| serde::de::Error::missing_field("type"))?;

                match event_type.as_str() {
                    "Thought" => Ok(ZhenfaStreamingEvent::Thought(
                        text.ok_or_else(|| serde::de::Error::missing_field("text"))?,
                    )),
                    "TextDelta" => Ok(ZhenfaStreamingEvent::TextDelta(
                        text.ok_or_else(|| serde::de::Error::missing_field("text"))?,
                    )),
                    "ToolCall" => Ok(ZhenfaStreamingEvent::ToolCall {
                        id: id.ok_or_else(|| serde::de::Error::missing_field("id"))?,
                        name: name.ok_or_else(|| serde::de::Error::missing_field("name"))?,
                        input: input.unwrap_or(Value::Null),
                    }),
                    "ToolResult" => Ok(ZhenfaStreamingEvent::ToolResult {
                        id: id.ok_or_else(|| serde::de::Error::missing_field("id"))?,
                        output: output.unwrap_or(Value::Null),
                    }),
                    "Status" => Ok(ZhenfaStreamingEvent::Status(
                        text.ok_or_else(|| serde::de::Error::missing_field("text"))?,
                    )),
                    "Progress" => Ok(ZhenfaStreamingEvent::Progress {
                        message: message
                            .ok_or_else(|| serde::de::Error::missing_field("message"))?,
                        percent: percent
                            .ok_or_else(|| serde::de::Error::missing_field("percent"))?,
                    }),
                    "Finished" => Ok(ZhenfaStreamingEvent::Finished(
                        outcome.ok_or_else(|| serde::de::Error::missing_field("outcome"))?,
                    )),
                    "Error" => Ok(ZhenfaStreamingEvent::Error {
                        code: code.ok_or_else(|| serde::de::Error::missing_field("code"))?,
                        message: message
                            .ok_or_else(|| serde::de::Error::missing_field("message"))?,
                    }),
                    _ => Err(serde::de::Error::custom(format!(
                        "unknown event type: {event_type}"
                    ))),
                }
            }
        }

        deserializer.deserialize_map(ZhenfaStreamingEventVisitor)
    }
}

impl ZhenfaStreamingEvent {
    /// Extract text content if this is a text-bearing event.
    #[must_use]
    pub fn text_content(&self) -> Option<&str> {
        match self {
            Self::Thought(text) | Self::TextDelta(text) | Self::Status(text) => Some(text),
            Self::Progress { message, .. } => Some(message),
            _ => None,
        }
    }
}
