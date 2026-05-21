//! LLM runtime primitives.

/// Unified acceleration mode parsing and config resolution.
pub mod acceleration;
/// Core LLM client traits and HTTP implementations.
pub mod client;
/// Structured LLM error model with user-safe sanitization.
pub mod error;
/// Backend mode parsing and normalized backend kinds.
#[path = "backend.rs"]
pub mod llm_backend;
/// Platform-agnostic multimodal marker parsing utilities.
pub mod multimodal;
/// Provider builders shared by runtime facades.
pub mod providers;
/// Runtime profile resolution for OpenAI-compatible multi-provider configs.
pub mod runtime_profile;
/// Vision preprocessing and semantic grounding utilities.
pub mod vision;

pub use client::{
    ChatChoice, ChatMessage, ChatRequest, ChatResponse, ContentPart, ImageUrlContent, LlmClient,
    MessageContent, OpenAIClient, OpenAICompatibleClient, OpenAIWireApi,
};
pub(crate) use error::sanitize_user_visible;
pub use error::{HttpContentType, LlmError, LlmResult};
pub use llm_backend::{LlmBackendKind, parse_llm_backend_kind};
#[cfg(feature = "provider-litellm")]
pub use multimodal::{Base64ImageSource, ImageMediaType, resolve_image_source_to_base64};
pub use runtime_profile::{
    LlmProviderProfileInput, LlmRuntimeDefaults, LlmRuntimeProfileEnv, LlmRuntimeProfileInput,
    ResolvedLlmRuntimeProfile, resolve_openai_runtime_profile,
};
