use super::types::{AgentConfig, ContextBudgetStrategy};

pub(super) fn default_max_tool_rounds() -> u32 {
    30
}

pub(super) fn default_tool_pool_size() -> usize {
    4
}

pub(super) fn default_tool_handshake_timeout_secs() -> u64 {
    30
}

pub(super) fn default_tool_connect_retries() -> u32 {
    3
}

pub(super) fn default_tool_strict_startup() -> bool {
    true
}

pub(super) fn default_tool_connect_retry_backoff_ms() -> u64 {
    1_000
}

pub(super) fn default_tool_timeout_secs() -> u64 {
    180
}

pub(super) fn default_tool_list_cache_ttl_ms() -> u64 {
    1_000
}

pub(super) fn default_consolidation_take_turns() -> usize {
    10
}

pub(super) fn default_consolidation_async() -> bool {
    true
}

pub(super) fn default_context_budget_reserve_tokens() -> usize {
    512
}

pub(super) fn default_summary_max_segments() -> usize {
    8
}

pub(super) fn default_summary_max_chars() -> usize {
    480
}

use crate::config::{RuntimeSettings, load_runtime_settings};

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            inference_url: "https://api.openai.com/v1/chat/completions".to_string(),
            model: "gpt-4o-mini".to_string(),
            api_key: None,
            tool_servers: Vec::new(),
            tool_pool_size: default_tool_pool_size(),
            tool_handshake_timeout_secs: default_tool_handshake_timeout_secs(),
            tool_connect_retries: default_tool_connect_retries(),
            tool_strict_startup: default_tool_strict_startup(),
            tool_connect_retry_backoff_ms: default_tool_connect_retry_backoff_ms(),
            tool_timeout_secs: default_tool_timeout_secs(),
            tool_list_cache_ttl_ms: default_tool_list_cache_ttl_ms(),
            max_tool_rounds: default_max_tool_rounds(),
            memory: None,
            window_max_turns: None,
            consolidation_threshold_turns: None,
            consolidation_take_turns: default_consolidation_take_turns(),
            consolidation_async: default_consolidation_async(),
            context_budget_tokens: None,
            context_budget_reserve_tokens: default_context_budget_reserve_tokens(),
            context_budget_strategy: ContextBudgetStrategy::default(),
            summary_max_segments: default_summary_max_segments(),
            summary_max_chars: default_summary_max_chars(),
        }
    }
}

/// Default `LiteLLM` proxy path (when using `litellm --port 4000`).
pub const LITELLM_DEFAULT_URL: &str = "http://localhost:4000/v1/chat/completions";

impl AgentConfig {
    /// Build config that uses a `LiteLLM` proxy as the inference endpoint.
    pub fn litellm(model: impl Into<String>) -> Self {
        let inference_url =
            std::env::var("LITELLM_PROXY_URL").unwrap_or_else(|_| LITELLM_DEFAULT_URL.to_string());
        let model = std::env::var("XIUXIAN_DAOCHANG_MODEL").unwrap_or_else(|_| model.into());
        Self {
            inference_url,
            model,
            api_key: None,
            tool_servers: Vec::new(),
            tool_pool_size: default_tool_pool_size(),
            tool_handshake_timeout_secs: default_tool_handshake_timeout_secs(),
            tool_connect_retries: default_tool_connect_retries(),
            tool_strict_startup: default_tool_strict_startup(),
            tool_connect_retry_backoff_ms: default_tool_connect_retry_backoff_ms(),
            tool_timeout_secs: default_tool_timeout_secs(),
            tool_list_cache_ttl_ms: default_tool_list_cache_ttl_ms(),
            max_tool_rounds: default_max_tool_rounds(),
            memory: None,
            window_max_turns: None,
            consolidation_threshold_turns: None,
            consolidation_take_turns: default_consolidation_take_turns(),
            consolidation_async: default_consolidation_async(),
            context_budget_tokens: None,
            context_budget_reserve_tokens: default_context_budget_reserve_tokens(),
            context_budget_strategy: ContextBudgetStrategy::default(),
            summary_max_segments: default_summary_max_segments(),
            summary_max_chars: default_summary_max_chars(),
        }
    }

    /// Resolve API key: config value, or env (`OPENAI_API_KEY` / `ANTHROPIC_API_KEY`).
    /// When inference goes to our own loopback tool/inference gateway, returns None
    /// so we do not send a key — the local service holds the key and forwards to the real LLM.
    #[must_use]
    pub fn resolve_api_key(&self) -> Option<String> {
        let runtime_settings = load_runtime_settings();
        self.resolve_api_key_with_runtime_settings_and_env_reader(&runtime_settings, |key| {
            std::env::var(key).ok()
        })
    }

    /// Resolve API key using a pluggable environment reader.
    ///
    /// This keeps runtime behavior identical while allowing deterministic tests
    /// without mutating process-wide environment variables.
    #[must_use]
    pub fn resolve_api_key_with_env_reader<F>(&self, mut read_env: F) -> Option<String>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let runtime_settings = load_runtime_settings();
        self.resolve_api_key_with_runtime_settings_and_env_reader(&runtime_settings, |key| {
            read_env(key)
        })
    }

    fn resolve_api_key_with_runtime_settings_and_env_reader<F>(
        &self,
        runtime_settings: &RuntimeSettings,
        mut read_env: F,
    ) -> Option<String>
    where
        F: FnMut(&str) -> Option<String>,
    {
        if let Some(ref key) = self.api_key {
            return Some(key.clone());
        }
        if self.inference_url.contains("127.0.0.1") || self.inference_url.contains("localhost") {
            return None;
        }
        if inference_url_matches_runtime_settings_base(&self.inference_url, runtime_settings) {
            return runtime_settings
                .inference
                .api_key
                .as_deref()
                .and_then(|configured| {
                    resolve_runtime_settings_api_key(configured, &mut read_env)
                });
        }
        if self.inference_url.contains("anthropic")
            || self.inference_url.contains("claude")
            || self.inference_url.contains("/messages")
        {
            return read_env("ANTHROPIC_API_KEY").or_else(|| read_env("ANTHROPIC_AUTH_TOKEN"));
        }
        read_env("OPENAI_API_KEY")
    }
}

fn inference_url_matches_runtime_settings_base(
    inference_url: &str,
    runtime_settings: &RuntimeSettings,
) -> bool {
    let Some(configured_base) = runtime_settings
        .inference
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return false;
    };

    canonicalize_runtime_api_base(inference_url) == canonicalize_runtime_api_base(configured_base)
}

fn canonicalize_runtime_api_base(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    let without_suffix = trimmed
        .strip_suffix("/v1/chat/completions")
        .or_else(|| trimmed.strip_suffix("/chat/completions"))
        .or_else(|| trimmed.strip_suffix("/v1/messages"))
        .or_else(|| trimmed.strip_suffix("/messages"))
        .unwrap_or(trimmed)
        .trim_end_matches('/');

    without_suffix
        .strip_suffix("/v1")
        .unwrap_or(without_suffix)
        .trim_end_matches('/')
        .to_string()
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

fn resolve_runtime_settings_api_key(
    configured: &str,
    read_env: &mut impl FnMut(&str) -> Option<String>,
) -> Option<String> {
    let raw = configured.trim();
    if raw.is_empty() {
        return None;
    }
    if let Some(env_name) = raw.strip_prefix("env:")
        && is_env_var_name(env_name)
    {
        return read_env(env_name);
    }
    if raw.starts_with("${")
        && raw.ends_with('}')
        && raw.len() > 3
        && is_env_var_name(&raw[2..raw.len() - 1])
    {
        return read_env(&raw[2..raw.len() - 1]);
    }
    if is_env_var_name(raw) {
        return read_env(raw);
    }
    Some(raw.to_string())
}
