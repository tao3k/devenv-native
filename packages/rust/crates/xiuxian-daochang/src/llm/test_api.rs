use anyhow::Result;
#[cfg(feature = "agent-provider-litellm")]
use litellm_rs::core::types::chat::ChatMessage as LiteChatMessage;
#[cfg(feature = "agent-provider-litellm")]
use litellm_rs::core::types::chat::ChatRequest as LiteChatRequest;
#[cfg(feature = "agent-provider-litellm")]
use litellm_rs::core::types::tools::ToolChoice as LiteToolChoice;
#[cfg(feature = "agent-provider-litellm")]
use std::collections::HashMap;
#[cfg(feature = "agent-provider-litellm")]
use xiuxian_llm::llm::providers::AnthropicCustomBaseTransport;

use crate::config::RuntimeSettings;
use crate::session::ChatMessage;

/// Test-facing backend mode mirror.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmBackendMode {
    /// OpenAI-compatible HTTP backend mode.
    OpenAiCompatibleHttp,
    /// `litellm-rs` backend mode.
    LiteLlmRs,
}

/// Test-facing prepared tool payload snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedTool {
    /// Tool name.
    pub name: String,
    /// Optional tool description.
    pub description: Option<String>,
    /// Optional tool schema parameters.
    pub parameters: Option<serde_json::Value>,
}

/// Test-facing request body for OpenAI-compatible chat completions.
#[derive(Debug, Clone)]
pub struct ChatCompletionRequest {
    /// Model identifier.
    pub model: String,
    /// Chat message list.
    pub messages: Vec<ChatMessage>,
    /// Optional max token cap.
    pub max_tokens: Option<u32>,
    /// Optional JSON tool definitions.
    pub tools: Option<Vec<serde_json::Value>>,
    /// Optional tool-choice string.
    pub tool_choice: Option<String>,
}

/// Test-facing provider mode mirror.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteLlmProviderMode {
    /// `OpenAI` provider mode.
    OpenAi,
    /// `MiniMax` provider mode.
    Minimax,
    /// Anthropic provider mode.
    Anthropic,
}

/// Test-facing wire protocol mirror.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteLlmWireApi {
    /// `/v1/chat/completions` payload protocol.
    ChatCompletions,
    /// `/v1/responses` payload protocol.
    Responses,
}

/// Test-facing provider resolution snapshot.
#[derive(Debug, Clone)]
pub struct ProviderSettings {
    /// Resolved provider mode.
    pub mode: LiteLlmProviderMode,
    /// Resolved wire protocol.
    pub wire_api: LiteLlmWireApi,
    /// Resolution source (`env`, `settings`, or `default`).
    pub source: &'static str,
    /// Literal API key when configured directly.
    pub api_key: Option<String>,
    /// API key environment variable name.
    pub api_key_env: String,
    /// `MiniMax` API base override.
    pub minimax_api_base: String,
    /// Resolved model string.
    pub model: String,
    /// Timeout in seconds.
    pub timeout_secs: u64,
    /// Optional request max tokens.
    pub max_tokens: Option<u32>,
    /// Optional request in-flight cap.
    pub max_in_flight: Option<usize>,
}

/// Test-facing tool message integrity report.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ToolMessageIntegrityReport {
    /// Number of assistant messages dropped due to incomplete tool chains.
    pub incomplete_assistants: usize,
    /// Number of tool messages dropped because they were linked to incomplete assistants.
    pub linked_tools: usize,
    /// Number of tool messages dropped because they were orphaned.
    pub orphan_tools: usize,
    /// Number of assistant messages dropped because tool call IDs were empty.
    pub empty_tool_call_assistants: usize,
}

impl ToolMessageIntegrityReport {
    /// Total count of dropped messages caused by integrity enforcement.
    #[must_use]
    pub fn dropped_total(self) -> usize {
        self.incomplete_assistants
            .saturating_add(self.linked_tools)
            .saturating_add(self.orphan_tools)
            .saturating_add(self.empty_tool_call_assistants)
    }
}

/// Default `OpenAI` API key environment variable name.
pub const DEFAULT_OPENAI_KEY_ENV: &str = super::providers::DEFAULT_OPENAI_KEY_ENV;
/// Default `MiniMax` API key environment variable name.
pub const DEFAULT_MINIMAX_KEY_ENV: &str = super::providers::DEFAULT_MINIMAX_KEY_ENV;
/// Default Anthropic API key environment variable name.
pub const DEFAULT_ANTHROPIC_KEY_ENV: &str = super::providers::DEFAULT_ANTHROPIC_KEY_ENV;

/// Parse backend mode from optional runtime value.
#[must_use]
pub fn parse_backend_mode(raw: Option<&str>) -> LlmBackendMode {
    match super::backend::parse_backend_mode(raw) {
        super::backend::LlmBackendMode::OpenAiCompatibleHttp => {
            LlmBackendMode::OpenAiCompatibleHttp
        }
        super::backend::LlmBackendMode::LiteLlmRs => LlmBackendMode::LiteLlmRs,
    }
}

/// Resolve the effective backend mode for a concrete inference URL.
#[must_use]
pub fn resolve_backend_mode_for_inference_url(
    runtime_settings: &RuntimeSettings,
    inference_url: &str,
    env_backend_raw: Option<&str>,
) -> (LlmBackendMode, &'static str) {
    let (mode, source) = super::client::test_resolve_backend_mode_for_inference_url(
        runtime_settings,
        inference_url,
        env_backend_raw,
    );
    let mode = match mode {
        super::backend::LlmBackendMode::OpenAiCompatibleHttp => {
            LlmBackendMode::OpenAiCompatibleHttp
        }
        super::backend::LlmBackendMode::LiteLlmRs => LlmBackendMode::LiteLlmRs,
    };
    (mode, source)
}

/// Extract API base URL from normalized inference endpoint URL.
#[must_use]
pub fn extract_api_base_from_inference_url(inference_url: &str) -> String {
    super::backend::extract_api_base_from_inference_url(inference_url)
}

/// Returns whether `OpenAI` requests should use OpenAI-compatible transport
/// instead of strict official `OpenAI` transport for the provided base URL.
#[must_use]
pub fn should_use_openai_like_for_base(api_base: &str) -> bool {
    xiuxian_llm::llm::providers::should_use_openai_like_for_base(api_base)
}

/// Returns whether OpenAI-compatible transport error indicates stream-only mode.
#[must_use]
pub fn is_openai_like_stream_required_error(rendered: &str) -> bool {
    xiuxian_llm::llm::providers::is_openai_like_stream_required_error_message(rendered)
}

/// Parse tool JSON definitions into test-facing snapshots.
#[must_use]
pub fn parse_tools_json(tools_json: Option<Vec<serde_json::Value>>) -> Vec<ParsedTool> {
    super::tools::parse_tools_json(tools_json)
        .into_iter()
        .map(|tool| ParsedTool {
            name: tool.name,
            description: tool.description,
            parameters: tool.parameters,
        })
        .collect()
}

/// Serialize a test chat-completion request into JSON value.
///
/// # Errors
///
/// Returns an error if serialization fails.
pub fn chat_completion_request_to_value(
    request: ChatCompletionRequest,
) -> Result<serde_json::Value> {
    let tools = request.tools.map(|tools| {
        tools
            .into_iter()
            .filter_map(|tool| {
                let name = tool.get("name")?.as_str()?.trim().to_string();
                if name.is_empty() {
                    return None;
                }
                let description = tool
                    .get("description")
                    .and_then(|d| d.as_str())
                    .map(str::trim)
                    .filter(|d| !d.is_empty())
                    .map(ToString::to_string);
                let parameters = tool
                    .get("input_schema")
                    .cloned()
                    .or_else(|| tool.get("parameters").cloned());
                Some(super::types::ToolDef {
                    typ: "function".to_string(),
                    function: super::types::FunctionDef {
                        name,
                        description,
                        parameters,
                    },
                })
            })
            .collect::<Vec<_>>()
    });

    let internal = super::types::ChatCompletionRequest {
        model: request.model,
        messages: request.messages,
        max_tokens: request.max_tokens,
        tools,
        tool_choice: request.tool_choice,
    };
    serde_json::to_value(&internal).map_err(Into::into)
}

/// Resolve provider settings with test-supplied env overrides.
#[must_use]
pub fn resolve_provider_settings_with_env(
    runtime_settings: &RuntimeSettings,
    requested_model: String,
    env_provider_raw: Option<&str>,
    env_minimax_api_base_raw: Option<&str>,
) -> ProviderSettings {
    let settings = super::providers::mode::resolve_provider_settings_with_env(
        runtime_settings,
        requested_model,
        env_provider_raw,
        env_minimax_api_base_raw,
    );
    ProviderSettings {
        mode: map_provider_mode(settings.mode),
        wire_api: map_wire_api(settings.wire_api),
        source: settings.source,
        api_key: settings.api_key,
        api_key_env: settings.api_key_env,
        minimax_api_base: settings.minimax_api_base,
        model: settings.model,
        timeout_secs: settings.timeout_secs,
        max_tokens: settings.max_tokens,
        max_in_flight: settings.max_in_flight,
    }
}

/// Enforce assistant/tool message-chain integrity for OpenAI-compatible requests.
#[must_use]
pub fn enforce_tool_message_integrity(
    messages: Vec<ChatMessage>,
) -> (Vec<ChatMessage>, ToolMessageIntegrityReport) {
    let (messages, report) = super::client::enforce_tool_message_integrity_for_tests(messages);
    (
        messages,
        ToolMessageIntegrityReport {
            incomplete_assistants: report.incomplete_assistants,
            linked_tools: report.linked_tools,
            orphan_tools: report.orphan_tools,
            empty_tool_call_assistants: report.empty_tool_call_assistants,
        },
    )
}

/// Test-facing custom-base fallback transport selector.
#[cfg(feature = "agent-provider-litellm")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomBaseFallbackTransport {
    /// OpenAI-compatible chat/completions transport.
    OpenAi,
    /// `MiniMax` OpenAI-compatible transport.
    Minimax,
    /// Anthropic `/v1/messages` bypass transport.
    AnthropicMessagesBypass,
}

#[cfg(feature = "agent-provider-litellm")]
/// Convert one chat message into a `litellm-rs` message payload.
///
/// # Errors
///
/// Returns an error when role/content conversion fails.
pub fn chat_message_to_litellm_message(message: ChatMessage) -> Result<LiteChatMessage> {
    super::converters::chat_message_to_litellm_message(message)
}

#[cfg(feature = "agent-provider-litellm")]
/// Convert one chat message into an OpenAI-compatible `litellm-rs` chat payload.
///
/// # Errors
///
/// Returns an error when role/content conversion fails.
pub fn chat_message_to_litellm_message_for_openai_chat(
    message: ChatMessage,
) -> Result<LiteChatMessage> {
    super::converters::chat_message_to_litellm_message_for_openai_chat(message)
}

/// Build an `OpenAI` `/responses` payload from test-facing chat inputs.
///
/// # Errors
///
/// Returns an error when chat message conversion fails.
#[cfg(feature = "agent-provider-litellm")]
pub fn build_responses_payload_from_chat_completion_request(
    request: ChatCompletionRequest,
) -> Result<serde_json::Value> {
    let ChatCompletionRequest {
        model,
        messages,
        max_tokens,
        tools,
        tool_choice,
    } = request;
    let tools = super::tools::parse_tools_json(tools)
        .into_iter()
        .map(|tool| tool.to_litellm_tool())
        .collect::<Vec<_>>();
    let lite_request = LiteChatRequest {
        model,
        messages: messages
            .into_iter()
            .map(super::converters::chat_message_to_litellm_message)
            .collect::<Result<Vec<_>>>()?,
        max_tokens,
        tools: (!tools.is_empty()).then_some(tools),
        tool_choice: tool_choice.map(LiteToolChoice::String),
        ..Default::default()
    };
    Ok(super::compat::litellm::build_responses_payload_for_tests(
        &lite_request,
    ))
}

/// Parse an `OpenAI` `/responses` SSE payload into tool names after alias remapping.
///
/// # Errors
///
/// Returns an error if stream parsing fails.
#[cfg(feature = "agent-provider-litellm")]
pub fn parse_responses_stream_tool_names(
    raw: &str,
    alias_to_original_tool_name: &[(String, String)],
) -> Result<Vec<String>> {
    let alias_to_original_tool_name = alias_to_original_tool_name
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
    super::compat::litellm::parse_responses_stream_tool_names_for_tests(
        raw,
        &alias_to_original_tool_name,
    )
}

/// Resolve transport-specific API key precedence for anthropic custom-base fallback.
#[cfg(feature = "agent-provider-litellm")]
#[must_use]
pub fn resolve_custom_base_transport_api_key_from_values(
    transport: CustomBaseFallbackTransport,
    explicit_api_key: Option<&str>,
    configured_key: Option<&str>,
    openai_key: Option<&str>,
    minimax_key: Option<&str>,
    anthropic_key: Option<&str>,
) -> Option<String> {
    xiuxian_llm::llm::providers::resolve_custom_base_transport_api_key_from_values(
        map_custom_base_fallback_transport(transport),
        explicit_api_key,
        configured_key,
        openai_key,
        minimax_key,
        anthropic_key,
    )
}

fn map_provider_mode(mode: super::providers::mode::LiteLlmProviderMode) -> LiteLlmProviderMode {
    match mode {
        super::providers::mode::LiteLlmProviderMode::OpenAi => LiteLlmProviderMode::OpenAi,
        super::providers::mode::LiteLlmProviderMode::Minimax => LiteLlmProviderMode::Minimax,
        super::providers::mode::LiteLlmProviderMode::Anthropic => LiteLlmProviderMode::Anthropic,
    }
}

fn map_wire_api(mode: super::providers::mode::LiteLlmWireApi) -> LiteLlmWireApi {
    match mode {
        super::providers::mode::LiteLlmWireApi::ChatCompletions => LiteLlmWireApi::ChatCompletions,
        super::providers::mode::LiteLlmWireApi::Responses => LiteLlmWireApi::Responses,
    }
}

#[cfg(feature = "agent-provider-litellm")]
fn map_custom_base_fallback_transport(
    transport: CustomBaseFallbackTransport,
) -> AnthropicCustomBaseTransport {
    match transport {
        CustomBaseFallbackTransport::OpenAi => AnthropicCustomBaseTransport::OpenAi,
        CustomBaseFallbackTransport::Minimax => AnthropicCustomBaseTransport::Minimax,
        CustomBaseFallbackTransport::AnthropicMessagesBypass => {
            AnthropicCustomBaseTransport::AnthropicMessagesBypass
        }
    }
}
