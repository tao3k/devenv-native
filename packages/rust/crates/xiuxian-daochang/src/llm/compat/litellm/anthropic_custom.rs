use anyhow::Result;
use litellm_rs::core::types::chat::ChatRequest as LiteChatRequest;

use crate::llm::providers::DEFAULT_ANTHROPIC_KEY_ENV;
use crate::llm::types::AssistantMessage;
use crate::session::{FunctionCall, ToolCallOut};
use xiuxian_llm::llm::providers::{
    AnthropicParsedResponse, anthropic_messages_endpoint_from_base,
    execute_anthropic_messages_from_litellm_request_with_image_hook, resolve_api_key_with_env,
    resolve_positive_usize_env,
};

use super::LiteLlmDispatchConfig;

pub(super) async fn chat_anthropic_without_model_registry(
    config: &LiteLlmDispatchConfig<'_>,
    request: LiteChatRequest,
    transport_api_key: Option<String>,
) -> Result<AssistantMessage> {
    let api_key = transport_api_key
        .or_else(|| {
            resolve_api_key_with_env(
                config.api_key,
                config.litellm_api_key_env,
                DEFAULT_ANTHROPIC_KEY_ENV,
            )
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "missing anthropic api key; set {} or {}",
                config.litellm_api_key_env,
                DEFAULT_ANTHROPIC_KEY_ENV
            )
        })?;

    let client = reqwest::Client::builder()
        .http1_only()
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .connect_timeout(std::time::Duration::from_secs(
            config.timeout_secs.clamp(5, 60),
        ))
        .build()?;
    let endpoint = anthropic_messages_endpoint_from_base(config.inference_api_base);
    let attempts = anthropic_custom_network_attempts();
    let parsed = execute_anthropic_messages_from_litellm_request_with_image_hook(
        &client,
        endpoint.as_str(),
        api_key.as_str(),
        &request,
        attempts,
        |_source| async move { None },
    )
    .await
    .map_err(|error| {
        anyhow::anyhow!("litellm-rs anthropic chat completion failed (custom-base bypass): {error}")
    })?;
    Ok(assistant_message_from_anthropic_parsed(parsed))
}

fn anthropic_custom_network_attempts() -> usize {
    resolve_positive_usize_env("XIUXIAN_DAOCHANG_ANTHROPIC_BYPASS_NETWORK_ATTEMPTS", 3)
}

fn assistant_message_from_anthropic_parsed(parsed: AnthropicParsedResponse) -> AssistantMessage {
    let tool_calls = parsed
        .tool_uses
        .into_iter()
        .map(|call| ToolCallOut {
            id: call.id,
            typ: "function".to_string(),
            function: FunctionCall {
                name: call.name,
                arguments: call.input.to_string(),
            },
        })
        .collect::<Vec<_>>();

    AssistantMessage {
        content: parsed.text,
        tool_calls: if tool_calls.is_empty() {
            None
        } else {
            Some(tool_calls)
        },
    }
}
