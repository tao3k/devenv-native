use anyhow::Result;
use litellm_rs::core::types::chat::ChatRequest as LiteChatRequest;
use std::collections::HashMap;
use xiuxian_llm::llm::providers::{
    OpenAiResponsesAssistantOutput, build_openai_responses_payload, parse_openai_responses_stream,
};

use crate::llm::types::AssistantMessage;
use crate::session::{FunctionCall as SessionFunctionCall, ToolCallOut};

pub(in crate::llm) fn build_responses_payload_for_tests(
    request: &LiteChatRequest,
) -> serde_json::Value {
    build_openai_responses_payload(request).payload
}

pub(in crate::llm) fn parse_responses_stream_tool_names_for_tests(
    raw: &str,
    alias_to_original_tool_name: &HashMap<String, String>,
) -> Result<Vec<String>> {
    let parsed = parse_openai_responses_stream(raw, alias_to_original_tool_name)?;
    Ok(parsed
        .tool_calls
        .into_iter()
        .map(|tool_call| tool_call.function.name)
        .collect())
}

pub(super) fn assistant_from_parsed_openai_responses(
    parsed: OpenAiResponsesAssistantOutput,
) -> AssistantMessage {
    let tool_calls = if parsed.tool_calls.is_empty() {
        None
    } else {
        Some(
            parsed
                .tool_calls
                .into_iter()
                .map(|tool_call| ToolCallOut {
                    id: tool_call.id,
                    typ: tool_call.tool_type.to_string(),
                    function: SessionFunctionCall {
                        name: tool_call.function.name,
                        arguments: tool_call.function.arguments,
                    },
                })
                .collect(),
        )
    };
    AssistantMessage {
        content: parsed.content,
        tool_calls,
    }
}
