//! `LiteLLM` compatibility maps provider payloads, tool calls, and response metadata into the local chat contract.

use anyhow::Result;
use futures::StreamExt;
use litellm_rs::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use litellm_rs::core::types::chat::ChatRequest as LiteChatRequest;
use litellm_rs::core::types::context::RequestContext as LiteRequestContext;
use litellm_rs::core::types::responses::ChatChunk as LiteChatChunk;
use litellm_rs::core::types::responses::ChatResponse as LiteChatResponse;
use litellm_rs::core::types::tools::ToolChoice as LiteToolChoice;
use litellm_rs::core::types::tools::{FunctionCall as LiteFunctionCall, ToolCall as LiteToolCall};
use std::collections::{BTreeMap, HashMap};
use tokio::sync::OnceCell;

use super::anthropic_custom::chat_anthropic_without_model_registry;
use super::responses;
use crate::llm::providers::{
    DEFAULT_ANTHROPIC_KEY_ENV, DEFAULT_MINIMAX_KEY_ENV, DEFAULT_OPENAI_KEY_ENV,
    LiteLlmProviderMode, LiteLlmWireApi,
};
use crate::llm::{
    AssistantMessage, PreparedTool, chat_message_to_litellm_message,
    chat_message_to_litellm_message_for_openai_chat, content_from_litellm, tool_call_from_litellm,
};
use crate::session::ChatMessage;
use xiuxian_llm::llm::providers::{
    AnthropicCustomBaseTransport, LiteLlmAnthropicProvider, LiteLlmMinimaxProvider,
    LiteLlmOpenAILikeProvider, LiteLlmOpenAIProvider, anthropic_custom_base_transport_label,
    build_anthropic_provider, build_minimax_provider, build_openai_like_provider,
    build_openai_provider, execute_anthropic_custom_base_fallback,
    execute_openai_responses_request, inline_openai_compatible_image_urls,
    is_openai_like_stream_required_error_message, normalize_optional_base_override,
    resolve_api_key_with_env, resolve_required_api_key_with_env,
    should_bypass_anthropic_model_validation, should_use_openai_like_for_base,
    summarize_anthropic_custom_base_failures,
};
/// Dispatch settings for a single `litellm-rs` chat request.
#[derive(Clone, Copy)]
pub(in crate::llm) struct LiteLlmDispatchConfig<'a> {
    pub(in crate::llm) provider_mode: LiteLlmProviderMode,
    pub(in crate::llm) wire_api: LiteLlmWireApi,
    pub(in crate::llm) model: &'a str,
    pub(in crate::llm) max_tokens: Option<u32>,
    pub(in crate::llm) api_key: Option<&'a str>,
    pub(in crate::llm) litellm_api_key_env: &'a str,
    pub(in crate::llm) inference_api_base: &'a str,
    pub(in crate::llm) minimax_api_base: &'a str,
    pub(in crate::llm) timeout_secs: u64,
}

/// Runtime compatibility adapter that isolates `litellm-rs` provider lifecycle.
pub(in crate::llm) struct LiteLlmRuntime {
    openai: OnceCell<LiteLlmOpenAIProvider>,
    openai_like: OnceCell<LiteLlmOpenAILikeProvider>,
    minimax: OnceCell<LiteLlmMinimaxProvider>,
    anthropic: OnceCell<LiteLlmAnthropicProvider>,
}

impl LiteLlmRuntime {
    #[must_use]
    pub(in crate::llm) fn new() -> Self {
        Self {
            openai: OnceCell::const_new(),
            openai_like: OnceCell::const_new(),
            minimax: OnceCell::const_new(),
            anthropic: OnceCell::const_new(),
        }
    }

    pub(in crate::llm) async fn chat(
        &self,
        config: LiteLlmDispatchConfig<'_>,
        messages: Vec<ChatMessage>,
        tools: Vec<PreparedTool>,
    ) -> Result<AssistantMessage> {
        let request =
            Self::build_request(config, config.model, config.max_tokens, messages, &tools)?;
        match config.provider_mode {
            LiteLlmProviderMode::OpenAi => self.chat_openai(config, request).await,
            LiteLlmProviderMode::Minimax => self.chat_minimax(config, request, None).await,
            LiteLlmProviderMode::Anthropic => self.chat_anthropic(config, request).await,
        }
    }

    fn build_request(
        config: LiteLlmDispatchConfig<'_>,
        model: &str,
        max_tokens: Option<u32>,
        messages: Vec<ChatMessage>,
        tools: &[PreparedTool],
    ) -> Result<LiteChatRequest> {
        let tools = if tools.is_empty() {
            None
        } else {
            Some(tools.iter().map(PreparedTool::to_litellm_tool).collect())
        };
        Ok(LiteChatRequest {
            model: model.to_string(),
            messages: messages
                .into_iter()
                .map(|message| {
                    if should_encode_tool_result_parts(config) {
                        chat_message_to_litellm_message(message)
                    } else {
                        chat_message_to_litellm_message_for_openai_chat(message)
                    }
                })
                .collect::<Result<Vec<_>>>()?,
            tools: tools.clone(),
            tool_choice: tools
                .as_ref()
                .map(|_| LiteToolChoice::String("auto".to_string())),
            max_tokens,
            ..Default::default()
        })
    }

    async fn chat_openai(
        &self,
        config: LiteLlmDispatchConfig<'_>,
        request: LiteChatRequest,
    ) -> Result<AssistantMessage> {
        let api_key = resolve_api_key_with_env(
            config.api_key,
            config.litellm_api_key_env,
            DEFAULT_OPENAI_KEY_ENV,
        );
        let api_base = normalize_optional_base_override(Some(config.inference_api_base))
            .unwrap_or_else(|| config.inference_api_base.to_string());
        if should_use_openai_like_for_base(&api_base) {
            return self
                .chat_openai_like_custom_base(config, request, api_key)
                .await;
        }
        let api_base_for_openai = api_base.clone();
        let api_key_for_openai = api_key.clone();

        let provider = self
            .openai
            .get_or_try_init(move || async move {
                build_openai_provider(api_base_for_openai, api_key_for_openai, config.timeout_secs)
                    .await
            })
            .await?;

        let response = LLMProvider::chat_completion(provider, request, LiteRequestContext::new())
            .await
            .map_err(|e| anyhow::anyhow!("litellm-rs chat completion failed: {e}"))?;
        chat_response_to_assistant(response)
    }

    async fn chat_openai_like_custom_base(
        &self,
        config: LiteLlmDispatchConfig<'_>,
        request: LiteChatRequest,
        transport_api_key: Option<String>,
    ) -> Result<AssistantMessage> {
        validate_openai_like_dispatch(config)?;
        match config.wire_api {
            LiteLlmWireApi::ChatCompletions => {
                let provider = self
                    .openai_like
                    .get_or_try_init(|| async {
                        let api_key = transport_api_key.clone().or_else(|| {
                            resolve_api_key_with_env(
                                config.api_key,
                                DEFAULT_OPENAI_KEY_ENV,
                                config.litellm_api_key_env,
                            )
                        });
                        let api_base =
                            normalize_optional_base_override(Some(config.inference_api_base))
                                .unwrap_or_else(|| config.inference_api_base.to_string());
                        build_openai_like_provider(api_base, api_key, config.timeout_secs).await
                    })
                    .await?;

                match LLMProvider::chat_completion(
                    provider,
                    request.clone(),
                    LiteRequestContext::new(),
                )
                .await
                {
                    Ok(response) => chat_response_to_assistant(response),
                    Err(error) => {
                        let rendered = error.to_string();
                        if is_openai_like_stream_required_error_message(&rendered) {
                            tracing::warn!(
                                event = "agent.llm.litellm.openai_like.retry_streaming",
                                model = %config.model,
                                inference_api_base = %config.inference_api_base,
                                "OpenAI-compatible endpoint requires stream=true; retrying via streaming transport"
                            );
                            return self
                                .chat_openai_like_custom_base_streaming(provider, request)
                                .await;
                        }
                        Err(anyhow::anyhow!(
                            "litellm-rs openai_like chat completion failed: {rendered}"
                        ))
                    }
                }
            }
            LiteLlmWireApi::Responses => {
                self.chat_openai_like_custom_base_responses(config, request, transport_api_key)
                    .await
            }
        }
    }

    async fn chat_openai_like_custom_base_streaming(
        &self,
        provider: &LiteLlmOpenAILikeProvider,
        request: LiteChatRequest,
    ) -> Result<AssistantMessage> {
        let mut stream =
            LLMProvider::chat_completion_stream(provider, request, LiteRequestContext::new())
                .await
                .map_err(|error| {
                    anyhow::anyhow!("litellm-rs openai_like stream chat completion failed: {error}")
                })?;
        let mut chunks = Vec::new();
        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|error| {
                anyhow::anyhow!("litellm-rs openai_like stream chunk failed: {error}")
            })?;
            chunks.push(chunk);
        }
        assistant_from_openai_like_stream_chunks(chunks)
    }

    async fn chat_openai_like_custom_base_responses(
        &self,
        config: LiteLlmDispatchConfig<'_>,
        request: LiteChatRequest,
        transport_api_key: Option<String>,
    ) -> Result<AssistantMessage> {
        let api_key = transport_api_key.or_else(|| {
            resolve_api_key_with_env(
                config.api_key,
                DEFAULT_OPENAI_KEY_ENV,
                config.litellm_api_key_env,
            )
        });
        let api_base = normalize_optional_base_override(Some(config.inference_api_base))
            .unwrap_or_else(|| config.inference_api_base.to_string());
        let endpoint = format!("{}/responses", api_base.trim_end_matches('/'));
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .connect_timeout(std::time::Duration::from_secs(
                config.timeout_secs.clamp(5, 60),
            ))
            .build()
            .map_err(|error| anyhow::anyhow!("failed to build responses http client: {error}"))?;
        let parsed =
            execute_openai_responses_request(&client, &endpoint, api_key.as_deref(), &request)
                .await
                .map_err(|error| {
                    anyhow::anyhow!("litellm-rs openai_like responses failed: {error}")
                })?;
        Ok(responses::assistant_from_parsed_openai_responses(parsed))
    }

    async fn chat_minimax(
        &self,
        config: LiteLlmDispatchConfig<'_>,
        request: LiteChatRequest,
        transport_api_key: Option<String>,
    ) -> Result<AssistantMessage> {
        let provider = self
            .minimax
            .get_or_try_init(|| async {
                let api_key = if let Some(key) = transport_api_key.clone() {
                    key
                } else {
                    resolve_required_api_key_with_env(
                        config.api_key,
                        DEFAULT_MINIMAX_KEY_ENV,
                        DEFAULT_OPENAI_KEY_ENV,
                        "minimax",
                    )?
                };
                let api_base_override =
                    normalize_optional_base_override(Some(config.minimax_api_base));
                build_minimax_provider(api_base_override, api_key, config.timeout_secs).await
            })
            .await?;

        let response = LLMProvider::chat_completion(provider, request, LiteRequestContext::new())
            .await
            .map_err(|e| anyhow::anyhow!("litellm-rs minimax chat completion failed: {e}"))?;
        chat_response_to_assistant(response)
    }

    async fn chat_anthropic(
        &self,
        config: LiteLlmDispatchConfig<'_>,
        request: LiteChatRequest,
    ) -> Result<AssistantMessage> {
        if should_bypass_anthropic_model_validation(config.inference_api_base) {
            return self.chat_anthropic_custom_base(config, request).await;
        }
        self.chat_anthropic_official(config, request).await
    }

    async fn chat_anthropic_custom_base(
        &self,
        config: LiteLlmDispatchConfig<'_>,
        request: LiteChatRequest,
    ) -> Result<AssistantMessage> {
        let image_client = build_custom_base_image_client(config.timeout_secs)?;
        let request = inline_openai_compatible_image_urls(&image_client, &request)
            .await
            .map_err(|error| {
                anyhow::anyhow!(
                    "failed to inline image URLs for anthropic custom-base fallback: {error}"
                )
            })?;

        match execute_anthropic_custom_base_fallback(config.model, |transport| {
            let request = request.clone();
            async move {
                self.chat_anthropic_custom_base_transport(config, request, transport)
                    .await
            }
        })
        .await
        {
            Ok(message) => Ok(message),
            Err(failures) => {
                log_anthropic_custom_base_failures(config, failures.attempts());
                let attempts = failures.into_attempts();
                if attempts.is_empty() {
                    return Err(anyhow::anyhow!(
                        "anthropic custom-base fallback exhausted without attempts"
                    ));
                }
                let summary = summarize_anthropic_custom_base_failures(&attempts);
                Err(anyhow::anyhow!(
                    "litellm-rs anthropic custom-base fallback exhausted after {} attempt(s): {}",
                    attempts.len(),
                    summary
                ))
            }
        }
    }

    async fn chat_anthropic_custom_base_transport(
        &self,
        config: LiteLlmDispatchConfig<'_>,
        request: LiteChatRequest,
        transport: AnthropicCustomBaseTransport,
    ) -> Result<AssistantMessage> {
        let transport_api_key = resolve_custom_base_transport_api_key(
            transport,
            config.api_key,
            config.litellm_api_key_env,
        );
        match transport {
            AnthropicCustomBaseTransport::OpenAi => {
                self.chat_openai_like_custom_base(config, request, transport_api_key)
                    .await
            }
            AnthropicCustomBaseTransport::Minimax => {
                self.chat_minimax(config, request, transport_api_key).await
            }
            AnthropicCustomBaseTransport::AnthropicMessagesBypass => {
                chat_anthropic_without_model_registry(&config, request, transport_api_key).await
            }
        }
    }

    async fn chat_anthropic_official(
        &self,
        config: LiteLlmDispatchConfig<'_>,
        request: LiteChatRequest,
    ) -> Result<AssistantMessage> {
        let provider = self
            .anthropic
            .get_or_try_init(|| async {
                let api_key = resolve_required_api_key_with_env(
                    config.api_key,
                    config.litellm_api_key_env,
                    DEFAULT_ANTHROPIC_KEY_ENV,
                    "anthropic",
                )?;
                build_anthropic_provider(
                    config.inference_api_base.to_string(),
                    api_key,
                    config.timeout_secs,
                )
                .await
            })
            .await?;

        let response = LLMProvider::chat_completion(provider, request, LiteRequestContext::new())
            .await
            .map_err(map_official_anthropic_error)?;
        chat_response_to_assistant(response)
    }
}

fn should_encode_tool_result_parts(config: LiteLlmDispatchConfig<'_>) -> bool {
    matches!(config.provider_mode, LiteLlmProviderMode::Anthropic)
        || matches!(config.wire_api, LiteLlmWireApi::Responses)
}

fn build_custom_base_image_client(timeout_secs: u64) -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .connect_timeout(std::time::Duration::from_secs(timeout_secs.clamp(5, 60)))
        .build()
        .map_err(|error| anyhow::anyhow!("failed to build custom-base image client: {error}"))
}

fn log_anthropic_custom_base_failures(
    config: LiteLlmDispatchConfig<'_>,
    attempts: &[(AnthropicCustomBaseTransport, anyhow::Error)],
) {
    for (transport, error) in attempts {
        tracing::warn!(
            event = "agent.llm.litellm.anthropic_custom_base.fallback",
            fallback_provider = anthropic_custom_base_transport_label(*transport),
            model = %config.model,
            inference_api_base = %config.inference_api_base,
            error = %error,
            "Fallback transport failed for anthropic custom-base"
        );
    }
}

fn map_official_anthropic_error(error: impl std::fmt::Display) -> anyhow::Error {
    let rendered = error.to_string();
    if rendered.contains("Unsupported model:") {
        anyhow::anyhow!(
            "litellm-rs anthropic chat completion failed: {rendered}. \
Configured provider is `anthropic`; use an Anthropic-compatible model under \
`llm.providers.anthropic.model`, or switch `llm.default_provider` to a provider \
that supports this model."
        )
    } else {
        anyhow::anyhow!("litellm-rs anthropic chat completion failed: {rendered}")
    }
}

fn read_non_empty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn resolve_custom_base_transport_api_key(
    transport: AnthropicCustomBaseTransport,
    explicit_api_key: Option<&str>,
    configured_env: &str,
) -> Option<String> {
    let configured = read_non_empty_env(configured_env);
    let openai = read_non_empty_env(DEFAULT_OPENAI_KEY_ENV);
    let minimax = read_non_empty_env(DEFAULT_MINIMAX_KEY_ENV);
    let anthropic = read_non_empty_env(DEFAULT_ANTHROPIC_KEY_ENV);

    xiuxian_llm::llm::providers::resolve_custom_base_transport_api_key_from_values(
        xiuxian_llm::llm::providers::AnthropicTransportKeyResolution {
            transport,
            explicit_api_key: explicit_api_key
                .map(xiuxian_llm::llm::providers::ProviderApiKeyRef::new),
            configured_key: configured
                .as_deref()
                .map(xiuxian_llm::llm::providers::ProviderApiKeyRef::new),
            openai_key: openai
                .as_deref()
                .map(xiuxian_llm::llm::providers::ProviderApiKeyRef::new),
            minimax_key: minimax
                .as_deref()
                .map(xiuxian_llm::llm::providers::ProviderApiKeyRef::new),
            anthropic_key: anthropic
                .as_deref()
                .map(xiuxian_llm::llm::providers::ProviderApiKeyRef::new),
        },
    )
}

fn validate_openai_like_dispatch(config: LiteLlmDispatchConfig<'_>) -> Result<()> {
    if config.model.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "invalid openai_like dispatch: model is empty"
        ));
    }
    if config.inference_api_base.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "invalid openai_like dispatch: inference_api_base is empty"
        ));
    }
    Ok(())
}

#[derive(Default)]
struct StreamingToolCallAccumulator {
    id: Option<String>,
    tool_type: Option<String>,
    function_name: Option<String>,
    function_arguments: String,
}

fn assistant_from_openai_like_stream_chunks(
    chunks: Vec<LiteChatChunk>,
) -> Result<AssistantMessage> {
    let mut content = String::new();
    let mut tool_calls: BTreeMap<u32, StreamingToolCallAccumulator> = BTreeMap::new();

    for chunk in chunks {
        for choice in chunk.choices {
            if let Some(delta_content) = choice.delta.content {
                content.push_str(&delta_content);
            }
            if let Some(delta_tool_calls) = choice.delta.tool_calls {
                for delta in delta_tool_calls {
                    let entry = tool_calls.entry(delta.index).or_default();
                    if let Some(id) = delta.id {
                        entry.id = Some(id);
                    }
                    if let Some(tool_type) = delta.tool_type {
                        entry.tool_type = Some(tool_type);
                    }
                    if let Some(function) = delta.function {
                        if let Some(name) = function.name {
                            entry.function_name = Some(name);
                        }
                        if let Some(arguments) = function.arguments {
                            entry.function_arguments.push_str(&arguments);
                        }
                    }
                }
            }
        }
    }

    let content = if content.trim().is_empty() {
        None
    } else {
        Some(content)
    };

    let mut normalized_tool_calls = Vec::new();
    for (index, call) in tool_calls {
        let Some(function_name) = call.function_name else {
            continue;
        };
        normalized_tool_calls.push(LiteToolCall {
            id: call.id.unwrap_or_else(|| format!("call_{index}")),
            tool_type: call.tool_type.unwrap_or_else(|| "function".to_string()),
            function: LiteFunctionCall {
                name: function_name,
                arguments: call.function_arguments,
            },
        });
    }

    let tool_calls = if normalized_tool_calls.is_empty() {
        None
    } else {
        Some(
            normalized_tool_calls
                .into_iter()
                .map(tool_call_from_litellm)
                .collect(),
        )
    };

    if content.is_none() && tool_calls.is_none() {
        return Err(anyhow::anyhow!(
            "litellm-rs openai_like stream completed without content or tool calls"
        ));
    }

    Ok(AssistantMessage {
        content,
        tool_calls,
    })
}

pub(in crate::llm) fn build_responses_payload_for_tests(
    request: &LiteChatRequest,
) -> serde_json::Value {
    responses::build_responses_payload_for_tests(request)
}

pub(in crate::llm) fn parse_responses_stream_tool_names_for_tests(
    raw: &str,
    alias_to_original_tool_name: &HashMap<String, String>,
) -> Result<Vec<String>> {
    responses::parse_responses_stream_tool_names_for_tests(raw, alias_to_original_tool_name)
}

fn chat_response_to_assistant(response: LiteChatResponse) -> Result<AssistantMessage> {
    let choice = response
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("litellm-rs response has no choices"))?;
    Ok(AssistantMessage {
        content: content_from_litellm(choice.message.content),
        tool_calls: choice
            .message
            .tool_calls
            .map(|calls| calls.into_iter().map(tool_call_from_litellm).collect()),
    })
}
