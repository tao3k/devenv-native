use std::path::Path;

use crate::{
    RuntimeSettings, ToolServerEntry,
    env_parse::{
        parse_bool_from_env, parse_positive_u32_from_env, parse_positive_u64_from_env,
        parse_positive_usize_from_env,
    },
    load_tool_config,
};
use anyhow::Result;

use super::types::McpRuntimeOptions as ToolRuntimeOptions;

pub(crate) fn resolve_runtime_tool_servers(
    tool_config_path: &Path,
) -> Result<Vec<ToolServerEntry>> {
    Ok(load_tool_config(tool_config_path)?
        .into_iter()
        .filter(|entry| entry.url.is_some() || entry.command.is_some())
        .collect())
}

pub(crate) fn resolve_runtime_tool_options(
    runtime_settings: &RuntimeSettings,
) -> ToolRuntimeOptions {
    ToolRuntimeOptions {
        pool_size: parse_positive_usize_from_env("XIUXIAN_DAOCHANG_TOOL_POOL_SIZE")
            .or(runtime_settings.tool_runtime.pool_size)
            .unwrap_or(8),
        handshake_timeout_secs: parse_positive_u64_from_env(
            "XIUXIAN_DAOCHANG_TOOL_HANDSHAKE_TIMEOUT_SECS",
        )
        .or(runtime_settings.tool_runtime.handshake_timeout_secs)
        .unwrap_or(10),
        connect_retries: parse_positive_u32_from_env("XIUXIAN_DAOCHANG_TOOL_CONNECT_RETRIES")
            .or(runtime_settings.tool_runtime.connect_retries)
            .unwrap_or(2),
        strict_startup: parse_bool_from_env("XIUXIAN_DAOCHANG_TOOL_STRICT_STARTUP")
            .or(runtime_settings.tool_runtime.strict_startup)
            .unwrap_or(false),
        connect_retry_backoff_ms: parse_positive_u64_from_env(
            "XIUXIAN_DAOCHANG_TOOL_CONNECT_RETRY_BACKOFF_MS",
        )
        .or(runtime_settings.tool_runtime.connect_retry_backoff_ms)
        .unwrap_or(500),
        tool_timeout_secs: parse_positive_u64_from_env("XIUXIAN_DAOCHANG_TOOL_TIMEOUT_SECS")
            .or(runtime_settings.tool_runtime.tool_timeout_secs)
            .unwrap_or(30),
        list_tools_cache_ttl_ms: parse_positive_u64_from_env(
            "XIUXIAN_DAOCHANG_TOOL_LIST_TOOLS_CACHE_TTL_MS",
        )
        .or(runtime_settings.tool_runtime.list_tools_cache_ttl_ms)
        .unwrap_or(5_000),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/runtime_agent_factory/tools.rs"]
mod tests;
