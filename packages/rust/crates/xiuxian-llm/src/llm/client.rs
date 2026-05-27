//! LLM runtime primitives and clients.

#[cfg(feature = "provider-litellm")]
use super::error::HttpContentType;
#[cfg(feature = "provider-litellm")]
use super::error::sanitize_user_visible;
use super::error::{LlmError, LlmResult};
use async_trait::async_trait;
use futures::Stream;
#[cfg(feature = "provider-litellm")]
use futures::StreamExt;
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::traits::provider::llm_provider::trait_definition::LLMProvider;
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::types::chat::ChatRequest as LiteChatRequest;
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::types::content::ContentPart as LiteContentPart;
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::types::context::RequestContext as LiteRequestContext;
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::types::message::MessageContent as LiteMessageContent;
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::types::responses::ChatChunk as LiteChatChunk;
#[cfg(feature = "provider-litellm")]
use tracing::info;

use std::pin::Pin;

#[cfg(feature = "provider-litellm")]
use crate::llm::providers::{
    build_openai_like_provider, execute_openai_responses_request,
    is_openai_like_stream_required_error_message,
};

#[cfg(feature = "provider-litellm")]
pub use litellm_rs::core::types::chat::{ChatMessage, ChatRequest};
#[cfg(feature = "provider-litellm")]
pub use litellm_rs::core::types::content::{ContentPart, ImageUrl as ImageUrlContent};
#[cfg(feature = "provider-litellm")]
pub use litellm_rs::core::types::message::{MessageContent, MessageRole};
#[cfg(feature = "provider-litellm")]
pub use litellm_rs::core::types::responses::{ChatChoice, ChatResponse};

#[cfg(not(feature = "provider-litellm"))]
pub use provider_disabled_types::{
    ChatChoice, ChatMessage, ChatRequest, ChatResponse, ContentPart, ImageUrlContent,
    MessageContent, MessageRole,
};

/// Type alias for a boxed stream of string chunks.
pub type ChatStream = Pin<Box<dyn Stream<Item = LlmResult<String>> + Send>>;

/// The core trait for interacting with Large Language Models.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Execute a chat-completion request and return the first text answer.
    async fn chat(&self, request: ChatRequest) -> LlmResult<String>;

    /// Execute a streaming chat-completion request.
    ///
    /// Returns a stream of text chunks for real-time processing.
    /// This enables cognitive supervision and early-halt during generation.
    async fn chat_stream(&self, request: ChatRequest) -> LlmResult<ChatStream>;
}

/// Standard OpenAI-compatible HTTP client.
pub struct OpenAIClient {
    /// API key used for bearer-token authorization.
    pub api_key: String,
    /// Base URL for the OpenAI-compatible endpoint.
    pub base_url: String,
    /// Shared HTTP client.
    pub http: reqwest::Client,
}

/// OpenAI-compatible wire API mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAIWireApi {
    /// Use `/chat/completions`.
    ChatCompletions,
    /// Use `/responses`.
    Responses,
}

impl OpenAIWireApi {
    /// Parse a wire mode token.
    #[must_use]
    pub fn parse(raw: Option<&str>) -> Self {
        let normalized = raw
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_ascii_lowercase);
        match normalized.as_deref() {
            Some("responses") => Self::Responses,
            Some("chat_completions" | "chat-completions" | "chat" | "completions") => {
                Self::ChatCompletions
            }
            _ => Self::ChatCompletions,
        }
    }

    /// Stable wire mode token.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ChatCompletions => "chat_completions",
            Self::Responses => "responses",
        }
    }
}

/// OpenAI-compatible client supporting both `/chat/completions` and `/responses`.
pub struct OpenAICompatibleClient {
    /// API key used for bearer-token authorization.
    pub api_key: String,
    /// Base URL for the OpenAI-compatible endpoint.
    pub base_url: String,
    /// Selected transport wire mode.
    pub wire_api: OpenAIWireApi,
    /// Shared HTTP client.
    pub http: reqwest::Client,
}

impl OpenAICompatibleClient {
    #[cfg(feature = "provider-litellm")]
    async fn retry_chat_with_stream_transport(
        &self,
        lite_request: LiteChatRequest,
    ) -> LlmResult<String> {
        let (primary_base, fallback_base) =
            build_openai_like_base_candidates(self.base_url.as_str());
        let mut output = self
            .execute_chat_stream_once(primary_base.as_str(), lite_request.clone())
            .await;
        if should_retry_openai_like_v1_fallback(&output)
            && let Some(fallback_base) = fallback_base.as_deref()
        {
            info!(
                "Primary chat stream endpoint returned 404; retrying with OpenAI /v1 fallback: {}",
                fallback_base
            );
            output = self
                .execute_chat_stream_once(fallback_base, lite_request)
                .await;
        }
        output
    }

    #[cfg(feature = "provider-litellm")]
    async fn execute_chat_completion_once(
        &self,
        api_base: &str,
        request: LiteChatRequest,
    ) -> LlmResult<String> {
        const OPENAI_LIKE_CHAT_TIMEOUT_SECS: u64 = 90;
        let provider = build_openai_like_provider(
            api_base.to_string(),
            Some(self.api_key.clone()),
            OPENAI_LIKE_CHAT_TIMEOUT_SECS,
        )
        .await?;
        let response = LLMProvider::chat_completion(&provider, request, LiteRequestContext::new())
            .await
            .map_err(|error| map_litellm_openai_like_error("chat completion", error))?;
        parse_litellm_chat_response_text(&response)
    }

    #[cfg(feature = "provider-litellm")]
    async fn execute_chat_stream_once(
        &self,
        api_base: &str,
        request: LiteChatRequest,
    ) -> LlmResult<String> {
        const OPENAI_LIKE_CHAT_TIMEOUT_SECS: u64 = 90;
        let provider = build_openai_like_provider(
            api_base.to_string(),
            Some(self.api_key.clone()),
            OPENAI_LIKE_CHAT_TIMEOUT_SECS,
        )
        .await?;
        let mut stream =
            LLMProvider::chat_completion_stream(&provider, request, LiteRequestContext::new())
                .await
                .map_err(|error| map_litellm_openai_like_error("stream chat completion", error))?;

        let mut chunks = Vec::new();
        while let Some(item) = stream.next().await {
            let chunk =
                item.map_err(|error| map_litellm_openai_like_error("stream chunk", error))?;
            chunks.push(chunk);
        }
        parse_litellm_chat_stream_text(chunks)
    }

    #[cfg(feature = "provider-litellm")]
    async fn chat_completions_with_litellm(&self, request: ChatRequest) -> LlmResult<String> {
        let lite_request = request;
        let (primary_base, fallback_base) =
            build_openai_like_base_candidates(self.base_url.as_str());

        let mut output = self
            .execute_chat_completion_once(primary_base.as_str(), lite_request.clone())
            .await;
        if should_retry_openai_like_stream_transport(&output) {
            info!(
                "OpenAI-compatible /chat/completions endpoint requires stream=true; retrying with litellm-rs stream transport"
            );
            return self.retry_chat_with_stream_transport(lite_request).await;
        }
        if should_retry_openai_like_v1_fallback(&output)
            && let Some(fallback_base) = fallback_base.as_deref()
        {
            info!(
                "Primary chat completion endpoint returned 404; retrying with OpenAI /v1 fallback: {}",
                fallback_base
            );
            output = self
                .execute_chat_completion_once(fallback_base, lite_request.clone())
                .await;
            if should_retry_openai_like_stream_transport(&output) {
                info!(
                    "OpenAI-compatible /chat/completions endpoint requires stream=true after /v1 fallback; retrying with litellm-rs stream transport"
                );
                return self.retry_chat_with_stream_transport(lite_request).await;
            }
        }
        output
    }

    #[cfg(feature = "provider-litellm")]
    async fn chat_responses_with_litellm(&self, request: ChatRequest) -> LlmResult<String> {
        let lite_request = request;
        let (primary_base, fallback_base) =
            build_openai_like_base_candidates(self.base_url.as_str());
        let primary_endpoint = format!("{}/responses", primary_base.trim_end_matches('/'));

        let mut output = execute_openai_responses_request(
            &self.http,
            primary_endpoint.as_str(),
            Some(self.api_key.as_str()),
            &lite_request,
        )
        .await
        .and_then(|parsed| parsed.content.ok_or(LlmError::EmptyTextChoice));

        if should_retry_openai_like_v1_fallback(&output)
            && let Some(fallback_base) = fallback_base.as_deref()
        {
            let fallback_endpoint = format!("{}/responses", fallback_base.trim_end_matches('/'));
            info!(
                "Primary responses endpoint returned 404; retrying with OpenAI /v1 fallback: {}",
                fallback_endpoint
            );
            output = execute_openai_responses_request(
                &self.http,
                fallback_endpoint.as_str(),
                Some(self.api_key.as_str()),
                &lite_request,
            )
            .await
            .and_then(|parsed| parsed.content.ok_or(LlmError::EmptyTextChoice));
        }

        output
    }
}

#[cfg(feature = "provider-litellm")]
fn map_litellm_openai_like_error(stage: &'static str, error: impl std::fmt::Display) -> LlmError {
    let rendered = error.to_string();
    let reason = sanitize_user_visible(&rendered);
    let lower = rendered.to_ascii_lowercase();
    if lower.contains("status 404")
        || lower.contains("resource not found")
        || lower.contains("not found for openai_like")
    {
        return LlmError::RequestFailed {
            status: reqwest::StatusCode::NOT_FOUND,
            content_type: HttpContentType::new("application/json"),
            reason,
        };
    }
    LlmError::Internal {
        message: sanitize_user_visible(&format!(
            "litellm-rs openai_like {stage} failed: {rendered}"
        )),
    }
}

#[cfg(feature = "provider-litellm")]
fn parse_litellm_chat_stream_text(chunks: Vec<LiteChatChunk>) -> LlmResult<String> {
    let mut content = String::new();
    for chunk in chunks {
        for choice in chunk.choices {
            if let Some(delta) = choice.delta.content {
                content.push_str(delta.as_str());
            }
        }
    }
    let trimmed = content.trim();
    if trimmed.is_empty() {
        Err(LlmError::EmptyTextChoice)
    } else {
        Ok(trimmed.to_string())
    }
}

#[cfg(feature = "provider-litellm")]
fn parse_litellm_chat_response_text(
    response: &litellm_rs::core::types::responses::ChatResponse,
) -> LlmResult<String> {
    response
        .choices
        .first()
        .and_then(|choice| litellm_message_content_to_text(choice.message.content.as_ref()))
        .ok_or(LlmError::EmptyTextChoice)
}

#[cfg(feature = "provider-litellm")]
fn extract_litellm_chat_stream_chunk_text(chunk: &LiteChatChunk) -> String {
    chunk
        .choices
        .iter()
        .filter_map(|choice| choice.delta.content.as_deref())
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(feature = "provider-litellm")]
fn litellm_message_content_to_text(content: Option<&LiteMessageContent>) -> Option<String> {
    let content = content?;
    match content {
        LiteMessageContent::Text(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        LiteMessageContent::Parts(parts) => {
            let text = parts
                .iter()
                .filter_map(|part| match part {
                    LiteContentPart::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
    }
}

#[cfg(feature = "provider-litellm")]
fn should_retry_openai_like_stream_transport(result: &LlmResult<String>) -> bool {
    let Err(error) = result else {
        return false;
    };
    let hint = match error {
        LlmError::RequestFailed { reason, .. } => reason.as_str(),
        LlmError::Internal { message } => message.as_str(),
        _ => return false,
    };
    is_openai_like_stream_required_error_message(hint)
}

#[cfg(feature = "provider-litellm")]
fn should_retry_openai_like_v1_fallback(result: &LlmResult<String>) -> bool {
    matches!(
        result,
        Err(LlmError::RequestFailed { status, .. }) if *status == reqwest::StatusCode::NOT_FOUND
    )
}

#[async_trait]
impl LlmClient for OpenAIClient {
    async fn chat(&self, request: ChatRequest) -> LlmResult<String> {
        let client = OpenAICompatibleClient {
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            wire_api: OpenAIWireApi::ChatCompletions,
            http: self.http.clone(),
        };
        client.chat(request).await
    }

    async fn chat_stream(&self, request: ChatRequest) -> LlmResult<ChatStream> {
        let client = OpenAICompatibleClient {
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            wire_api: OpenAIWireApi::ChatCompletions,
            http: self.http.clone(),
        };
        client.chat_stream(request).await
    }
}

#[async_trait]
impl LlmClient for OpenAICompatibleClient {
    async fn chat(&self, request: ChatRequest) -> LlmResult<String> {
        #[cfg(feature = "provider-litellm")]
        match self.wire_api {
            OpenAIWireApi::ChatCompletions => {
                return self.chat_completions_with_litellm(request).await;
            }
            OpenAIWireApi::Responses => {
                return self.chat_responses_with_litellm(request).await;
            }
        }

        #[cfg(not(feature = "provider-litellm"))]
        let _ = request;
        #[cfg(not(feature = "provider-litellm"))]
        Err(LlmError::Internal {
            message: "OpenAICompatibleClient requires feature `provider-litellm`".to_string(),
        })
    }

    async fn chat_stream(&self, request: ChatRequest) -> LlmResult<ChatStream> {
        #[cfg(feature = "provider-litellm")]
        {
            let provider = build_openai_like_provider(
                self.base_url.clone(),
                Some(self.api_key.clone()),
                90, // timeout seconds
            )
            .await?;

            let stream =
                LLMProvider::chat_completion_stream(&provider, request, LiteRequestContext::new())
                    .await
                    .map_err(|e| map_litellm_openai_like_error("stream initiation", e))?;

            // Map the litellm stream to our ChatStream type
            let mapped = stream.map(|result| {
                result
                    .map_err(|e| map_litellm_openai_like_error("stream chunk", e))
                    .map(|chunk| extract_litellm_chat_stream_chunk_text(&chunk))
            });

            Ok(Box::pin(mapped))
        }

        #[cfg(not(feature = "provider-litellm"))]
        let _ = request;
        #[cfg(not(feature = "provider-litellm"))]
        Err(LlmError::Internal {
            message: "chat_stream requires feature `provider-litellm`".to_string(),
        })
    }
}

#[cfg(feature = "provider-litellm")]
fn build_openai_like_base_candidates(base_url: &str) -> (String, Option<String>) {
    let primary = base_url.trim_end_matches('/').to_string();
    if primary.ends_with("/v1") {
        return (primary, None);
    }

    let fallback = format!("{primary}/v1");
    if fallback == primary {
        (primary, None)
    } else {
        (primary, Some(fallback))
    }
}

#[cfg(not(feature = "provider-litellm"))]
#[path = "provider_disabled_types.rs"]
mod provider_disabled_types;

#[cfg(all(test, feature = "provider-litellm"))]
#[path = "../../tests/unit/llm/client.rs"]
mod tests;
