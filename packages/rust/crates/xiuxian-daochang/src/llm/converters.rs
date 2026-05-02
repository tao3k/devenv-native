use anyhow::Result;
use litellm_rs::core::types::chat::ChatMessage as LiteChatMessage;
use litellm_rs::core::types::content::{ContentPart as LiteContentPart, ImageUrl as LiteImageUrl};
use litellm_rs::core::types::message::{
    MessageContent as LiteMessageContent, MessageRole as LiteMessageRole,
};
use litellm_rs::core::types::tools::{FunctionCall as LiteFunctionCall, ToolCall as LiteToolCall};
use xiuxian_llm::llm::multimodal::{MultimodalContentPart, parse_multimodal_text_content};

use crate::session::{ChatMessage, FunctionCall, ToolCallOut};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::llm) enum ToolMessageEncoding {
    OpenAiCompatibleText,
    ToolResultParts,
}

fn to_litellm_role(raw_role: &str) -> Result<LiteMessageRole> {
    match raw_role {
        "system" => Ok(LiteMessageRole::System),
        "developer" => Ok(LiteMessageRole::Developer),
        "user" => Ok(LiteMessageRole::User),
        "assistant" => Ok(LiteMessageRole::Assistant),
        "tool" => Ok(LiteMessageRole::Tool),
        "function" => Ok(LiteMessageRole::Function),
        other => Err(anyhow::anyhow!(
            "unsupported chat role for litellm-rs: {other}"
        )),
    }
}

pub(crate) fn chat_message_to_litellm_message(message: ChatMessage) -> Result<LiteChatMessage> {
    chat_message_to_litellm_message_with_encoding(message, ToolMessageEncoding::ToolResultParts)
}

pub(crate) fn chat_message_to_litellm_message_for_openai_chat(
    message: ChatMessage,
) -> Result<LiteChatMessage> {
    chat_message_to_litellm_message_with_encoding(
        message,
        ToolMessageEncoding::OpenAiCompatibleText,
    )
}

fn chat_message_to_litellm_message_with_encoding(
    message: ChatMessage,
    tool_message_encoding: ToolMessageEncoding,
) -> Result<LiteChatMessage> {
    let ChatMessage {
        role,
        content,
        tool_calls,
        tool_call_id,
        name,
    } = message;
    Ok(LiteChatMessage {
        role: to_litellm_role(role.as_str())?,
        content: convert_message_content(
            role.as_str(),
            content.as_deref(),
            tool_call_id.as_deref(),
            tool_message_encoding,
        ),
        thinking: None,
        name,
        tool_calls: tool_calls.map(|calls| {
            calls
                .into_iter()
                .map(|call| LiteToolCall {
                    id: call.id,
                    tool_type: call.typ,
                    function: LiteFunctionCall {
                        name: call.function.name,
                        arguments: call.function.arguments,
                    },
                })
                .collect()
        }),
        tool_call_id,
        function_call: None,
    })
}

fn convert_message_content(
    role: &str,
    content: Option<&str>,
    tool_call_id: Option<&str>,
    tool_message_encoding: ToolMessageEncoding,
) -> Option<LiteMessageContent> {
    let content = content?;

    if role == "tool"
        && matches!(tool_message_encoding, ToolMessageEncoding::ToolResultParts)
        && let Some(tool_use_id) = tool_call_id
            .map(str::trim)
            .filter(|tool_use_id| !tool_use_id.is_empty())
    {
        return Some(LiteMessageContent::Parts(vec![
            LiteContentPart::ToolResult {
                tool_use_id: tool_use_id.to_string(),
                content: serde_json::Value::String(content.to_string()),
                is_error: None,
            },
        ]));
    }

    if let Some(parts) = parse_multimodal_text_content(content) {
        return Some(LiteMessageContent::Parts(
            parts.into_iter().map(multimodal_part_to_litellm).collect(),
        ));
    }

    Some(LiteMessageContent::Text(content.to_string()))
}

fn multimodal_part_to_litellm(part: MultimodalContentPart) -> LiteContentPart {
    match part {
        MultimodalContentPart::Text(text) => LiteContentPart::Text { text },
        MultimodalContentPart::ImageUrl { url } => LiteContentPart::ImageUrl {
            image_url: LiteImageUrl {
                url,
                detail: Some("high".to_string()),
            },
        },
    }
}

pub(crate) fn content_from_litellm(content: Option<LiteMessageContent>) -> Option<String> {
    match content {
        None => None,
        Some(LiteMessageContent::Text(text)) => Some(text),
        Some(LiteMessageContent::Parts(parts)) => {
            let text = parts
                .into_iter()
                .filter_map(|part| match part {
                    LiteContentPart::Text { text } => Some(text),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            if text.is_empty() { None } else { Some(text) }
        }
    }
}

pub(crate) fn tool_call_from_litellm(call: LiteToolCall) -> ToolCallOut {
    ToolCallOut {
        id: call.id,
        typ: call.tool_type,
        function: FunctionCall {
            name: call.function.name,
            arguments: call.function.arguments,
        },
    }
}
