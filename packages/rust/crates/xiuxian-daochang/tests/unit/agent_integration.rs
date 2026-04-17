//! Test coverage for xiuxian-daochang behavior.
//!
//! Integration test: Agent with runtime-default live LLM settings and optional
//! external tools. Enable with `XIUXIAN_DAOCHANG_LIVE_AGENT_INTEGRATION=1`,
//! and optionally point `OMNI_AGENT_TOOL_URL` at an external tool server.

use std::path::PathBuf;

use anyhow::Result;
use xiuxian_daochang::test_support::{resolve_runtime_inference_url, resolve_runtime_model};
use xiuxian_daochang::{Agent, AgentConfig, ToolServerEntry, load_runtime_settings_from_paths};

fn live_agent_integration_enabled() -> bool {
    std::env::var("XIUXIAN_DAOCHANG_LIVE_AGENT_INTEGRATION")
        .ok()
        .map(|raw| raw.trim().to_ascii_lowercase())
        .is_some_and(|raw| matches!(raw.as_str(), "1" | "true" | "yes" | "on"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .map_or_else(
            || panic!("crate dir should have repo root ancestor"),
            PathBuf::from,
        )
}

fn resolve_runtime_default_config() -> Result<Option<AgentConfig>> {
    let repo_root = repo_root();
    let system_settings =
        repo_root.join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user_settings = repo_root.join(".config/xiuxian-artisan-workshop/xiuxian.toml");
    let runtime_settings = load_runtime_settings_from_paths(&system_settings, &user_settings);
    let tool_servers = resolve_optional_tool_servers();
    let inference_url = resolve_runtime_inference_url(&runtime_settings, &tool_servers)?;
    let model = resolve_runtime_model(&runtime_settings);
    let api_key = runtime_settings
        .inference
        .api_key
        .as_deref()
        .and_then(resolve_configured_api_key);

    if model.trim().is_empty() {
        eprintln!("skip: runtime-default model is unresolved from repo xiuxian.toml");
        return Ok(None);
    }
    if api_key.is_none() {
        eprintln!("skip: runtime-default provider api key is unresolved from repo xiuxian.toml");
        return Ok(None);
    }

    Ok(Some(AgentConfig {
        inference_url,
        model,
        api_key,
        tool_servers,
        max_tool_rounds: 5,
        ..AgentConfig::default()
    }))
}

fn resolve_optional_tool_servers() -> Vec<ToolServerEntry> {
    std::env::var("OMNI_AGENT_TOOL_URL")
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
        .map(|url| ToolServerEntry {
            name: "live".to_string(),
            url: Some(url),
            command: None,
            args: None,
        })
        .into_iter()
        .collect()
}

fn resolve_configured_api_key(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(env_name) = trimmed.strip_prefix("env:")
        && is_env_var_name(env_name)
    {
        return std::env::var(env_name)
            .ok()
            .filter(|value| !value.trim().is_empty());
    }
    if trimmed.starts_with("${")
        && trimmed.ends_with('}')
        && trimmed.len() > 3
        && is_env_var_name(&trimmed[2..trimmed.len() - 1])
    {
        return std::env::var(&trimmed[2..trimmed.len() - 1])
            .ok()
            .filter(|value| !value.trim().is_empty());
    }
    if is_env_var_name(trimmed) {
        return std::env::var(trimmed)
            .ok()
            .filter(|value| !value.trim().is_empty());
    }
    Some(trimmed.to_string())
}

fn is_env_var_name(raw: &str) -> bool {
    let mut chars = raw.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_uppercase()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_uppercase() || ch.is_ascii_digit())
}

#[tokio::test]
async fn test_agent_one_turn_with_llm_and_tools() -> Result<()> {
    if !live_agent_integration_enabled() {
        return Ok(());
    }

    let Some(config) = resolve_runtime_default_config()? else {
        return Ok(());
    };

    let agent = Agent::from_config(config).await?;
    let output = agent
        .run_turn("test-session", "Say hello in one short sentence.")
        .await?;

    assert!(!output.trim().is_empty());
    Ok(())
}
