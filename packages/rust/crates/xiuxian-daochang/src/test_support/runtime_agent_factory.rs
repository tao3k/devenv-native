//! Runtime-agent factory helpers exposed for integration tests.
//!
//! These wrappers avoid `#[path = "../src/..."]` test remapping while keeping
//! runtime-agent implementation details internal to the crate.

use anyhow::Result;
use xiuxian_llm::embedding::EmbeddingBackendKind;

use crate::runtime_agent_factory;
use crate::{MemoryConfig, RuntimeSettings, ToolServerEntry};

/// Resolved runtime memory options used by tests.
#[derive(Debug, Clone)]
pub struct RuntimeMemoryResolution {
    /// Effective memory configuration after defaults and env overlays.
    pub config: MemoryConfig,
    /// Effective embedding backend mode.
    pub embedding_backend_mode: EmbeddingBackendKind,
}

/// Resolve inference URL from runtime env/config candidates.
#[must_use]
pub fn resolve_inference_url(
    litellm_proxy_url: Option<&str>,
    agent_inference_url: Option<&str>,
) -> String {
    runtime_agent_factory::resolve_inference_url(litellm_proxy_url, agent_inference_url)
}

/// Parse embedding backend mode from a raw runtime string.
#[must_use]
pub fn parse_embedding_backend_mode(raw: Option<&str>) -> Option<EmbeddingBackendKind> {
    runtime_agent_factory::parse_embedding_backend_mode(raw)
}

/// Resolve embedding backend mode from runtime settings and env overrides.
#[must_use]
pub fn resolve_runtime_embedding_backend_mode(
    runtime_settings: &RuntimeSettings,
) -> EmbeddingBackendKind {
    runtime_agent_factory::resolve_runtime_embedding_backend_mode(runtime_settings)
}

/// Resolve effective embedding base URL based on backend mode.
#[must_use]
pub fn resolve_runtime_embedding_base_url(
    runtime_settings: &RuntimeSettings,
    backend_mode: EmbeddingBackendKind,
) -> Option<String> {
    runtime_agent_factory::resolve_runtime_embedding_base_url(runtime_settings, backend_mode)
}

/// Validate that inference endpoint origin does not conflict with configured external tool servers.
///
/// # Errors
///
/// Returns an error when inference URL and external tool endpoints share the same origin
/// while shared-origin mode is disabled.
pub fn validate_inference_url_origin(
    inference_url: &str,
    tool_servers: &[ToolServerEntry],
    allow_shared_origin: bool,
) -> Result<()> {
    runtime_agent_factory::validate_inference_url_origin(
        inference_url,
        tool_servers,
        allow_shared_origin,
    )
}

/// Resolve runtime inference URL using runtime settings and external tool server list.
///
/// # Errors
///
/// Returns an error when the resulting inference endpoint violates runtime URL
/// origin constraints.
pub fn resolve_runtime_inference_url(
    runtime_settings: &RuntimeSettings,
    tool_servers: &[ToolServerEntry],
) -> Result<String> {
    runtime_agent_factory::resolve_runtime_inference_url(runtime_settings, tool_servers)
}

/// Resolve runtime model with env/config precedence.
#[must_use]
pub fn resolve_runtime_model(runtime_settings: &RuntimeSettings) -> String {
    runtime_agent_factory::resolve_runtime_model(runtime_settings)
}

/// Resolve effective runtime memory options.
#[must_use]
pub fn resolve_runtime_memory_options(
    runtime_settings: &RuntimeSettings,
) -> RuntimeMemoryResolution {
    let resolved = runtime_agent_factory::resolve_runtime_memory_options(runtime_settings);
    RuntimeMemoryResolution {
        config: resolved.config,
        embedding_backend_mode: resolved.embedding_backend_mode,
    }
}
