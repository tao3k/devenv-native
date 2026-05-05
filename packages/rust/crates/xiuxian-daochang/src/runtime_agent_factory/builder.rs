//! Runtime settings to agent configuration builder.

use std::path::Path;

use anyhow::Result;

use crate::{Agent, AgentConfig, RuntimeSettings};

use super::logging::log_runtime_agent_options;
use super::session::resolve_runtime_session_options;
use super::{
    resolve_runtime_inference_url, resolve_runtime_memory_options, resolve_runtime_model,
    resolve_runtime_tool_options, resolve_runtime_tool_servers,
};

/// Build an agent instance from runtime settings and the external tool config file.
///
/// # Errors
///
/// Returns an error when tool config loading, runtime option resolution, or
/// agent initialization fails.
pub async fn build_agent(
    tool_config_path: &Path,
    runtime_settings: &RuntimeSettings,
) -> Result<Agent> {
    let tool_servers = resolve_runtime_tool_servers(tool_config_path)?;
    let inference_url = resolve_runtime_inference_url(runtime_settings, &tool_servers)?;
    let model = resolve_runtime_model(runtime_settings);
    let tool_runtime = resolve_runtime_tool_options(runtime_settings);
    let session = resolve_runtime_session_options(runtime_settings)?;
    let memory = resolve_runtime_memory_options(runtime_settings);

    log_runtime_agent_options(&tool_runtime, &session, &memory);

    let config = AgentConfig {
        inference_url,
        model,
        api_key: None,
        tool_servers,
        tool_pool_size: tool_runtime.pool_size,
        tool_handshake_timeout_secs: tool_runtime.handshake_timeout_secs,
        tool_connect_retries: tool_runtime.connect_retries,
        tool_strict_startup: tool_runtime.strict_startup,
        tool_connect_retry_backoff_ms: tool_runtime.connect_retry_backoff_ms,
        tool_timeout_secs: tool_runtime.tool_timeout_secs,
        tool_list_cache_ttl_ms: tool_runtime.list_tools_cache_ttl_ms,
        max_tool_rounds: session.max_tool_rounds,
        memory: Some(memory.config),
        window_max_turns: session.window_max_turns,
        consolidation_threshold_turns: session.consolidation_threshold_turns,
        consolidation_take_turns: session.consolidation_take_turns,
        consolidation_async: session.consolidation_async,
        context_budget_tokens: session.context_budget_tokens,
        context_budget_reserve_tokens: session.context_budget_reserve_tokens,
        context_budget_strategy: session.context_budget_strategy,
        summary_max_segments: session.summary_max_segments,
        summary_max_chars: session.summary_max_chars,
    };
    Agent::from_config(config).await
}
