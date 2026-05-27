//! OpenAI-compatible provider construction and Responses API transport.

#[cfg(feature = "provider-litellm")]
mod facade;
#[cfg(feature = "provider-litellm")]
mod responses;

#[cfg(feature = "provider-litellm")]
pub use facade::{
    LiteLlmOpenAILikeProvider, build_openai_like_provider, execute_openai_chat_completions_request,
    execute_openai_responses_request, inline_openai_compatible_image_urls,
    is_openai_like_stream_required_error_message,
};
#[cfg(feature = "provider-litellm")]
pub use responses::{
    OpenAiResponsesAssistantOutput, OpenAiResponsesPayload, OpenAiResponsesToolCall,
    OpenAiResponsesToolType, build_openai_responses_payload, parse_openai_responses_stream,
    remap_openai_responses_tool_name,
};
