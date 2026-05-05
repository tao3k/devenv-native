//! Agent runtime configuration types.

use serde::{Deserialize, Serialize};

use super::{agent_defaults, memory_defaults};
use crate::config::{RuntimeSettings, load_runtime_settings};

/// One external tool server entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolServerEntry {
    /// Display name for logging.
    pub name: String,
    /// For Streamable HTTP: full URL (e.g. `http://localhost:3002/sse`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// For stdio: command to spawn.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// For stdio: arguments to the command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
}

/// Optional memory (xiuxian-memory-engine) config for two-phase recall and episode storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Path to the memory store (directory).
    #[serde(default = "memory_defaults::default_memory_path")]
    pub path: String,
    /// Optional embedding backend override for memory runtime.
    ///
    /// Supported values:
    /// - `http`: legacy `/embed/batch` endpoint
    /// - `openai_http`: generic OpenAI-compatible `/v1/embeddings` endpoint
    /// - `litellm_rs`: Rust `litellm-rs` provider path
    ///   (provider/API-key oriented; no-key mode stays on Rust HTTP paths)
    ///
    /// Default:
    /// - `litellm_rs` when feature `agent-provider-litellm` is enabled.
    /// - `http` when that feature is disabled.
    ///
    /// When unset, backend selection follows runtime settings / environment defaults.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_backend: Option<String>,
    /// Optional embedding client base URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_base_url: Option<String>,
    /// Optional embedding model id used by the embedding service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_model: Option<String>,
    /// Optional max input texts per embedding batch request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_batch_max_size: Option<usize>,
    /// Optional max concurrent embedding chunks per batch request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_batch_max_concurrency: Option<usize>,
    /// Optional per-attempt timeout (ms) for memory embedding calls.
    ///
    /// This timeout is used by semantic memory recall/store operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_timeout_ms: Option<u64>,
    /// Optional cooldown window (ms) after an embedding timeout.
    ///
    /// During cooldown, memory embedding requests are rejected quickly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_timeout_cooldown_ms: Option<u64>,
    /// Embedding dimension for intent vectors (must match encoder).
    #[serde(default = "memory_defaults::default_embedding_dim")]
    pub embedding_dim: usize,
    /// Table name for episodes.
    #[serde(default = "memory_defaults::default_memory_table")]
    pub table_name: String,
    /// Phase 1 candidate count for two-phase recall.
    #[serde(default = "memory_defaults::default_recall_k1")]
    pub recall_k1: usize,
    /// Phase 2 result count after Q-value reranking.
    #[serde(default = "memory_defaults::default_recall_k2")]
    pub recall_k2: usize,
    /// Q-value weight in reranking (0.0 = semantic only, 1.0 = Q only).
    #[serde(default = "memory_defaults::default_recall_lambda")]
    pub recall_lambda: f32,
    /// Persistence backend mode: auto/local/valkey.
    #[serde(default = "memory_defaults::default_memory_persistence_backend")]
    pub persistence_backend: String,
    /// Optional Valkey URL override injected by runtime builder/tests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence_valkey_url: Option<String>,
    /// Key prefix for Valkey-backed memory state.
    #[serde(default = "memory_defaults::default_memory_persistence_key_prefix")]
    pub persistence_key_prefix: String,
    /// Optional strict-startup override for Valkey-backed persistence.
    ///
    /// - `Some(true)`: fail startup when initial Valkey load fails.
    /// - `Some(false)`: continue startup with empty memory on load failure.
    /// - `None`: use backend defaults (strict for Valkey, relaxed for local).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence_strict_startup: Option<bool>,
    /// Whether to apply post-turn recall credit updates to recalled episodes.
    #[serde(default = "memory_defaults::default_recall_credit_enabled")]
    pub recall_credit_enabled: bool,
    /// Maximum recalled episodes to receive post-turn credit updates.
    #[serde(default = "memory_defaults::default_recall_credit_max_candidates")]
    pub recall_credit_max_candidates: usize,
    /// Whether to apply periodic memory decay.
    #[serde(default = "memory_defaults::default_decay_enabled")]
    pub decay_enabled: bool,
    /// Apply memory decay every N successful stored turns.
    #[serde(default = "memory_defaults::default_decay_every_turns")]
    pub decay_every_turns: usize,
    /// Decay factor passed to memory store decay routine.
    #[serde(default = "memory_defaults::default_decay_factor")]
    pub decay_factor: f32,
    /// Utility threshold for promote gate decision.
    #[serde(default = "memory_defaults::default_gate_promote_threshold")]
    pub gate_promote_threshold: f32,
    /// Utility threshold for obsolete gate decision.
    #[serde(default = "memory_defaults::default_gate_obsolete_threshold")]
    pub gate_obsolete_threshold: f32,
    /// Minimum usage count required before promote is allowed.
    #[serde(default = "memory_defaults::default_gate_promote_min_usage")]
    pub gate_promote_min_usage: u32,
    /// Minimum usage count required before obsolete is allowed.
    #[serde(default = "memory_defaults::default_gate_obsolete_min_usage")]
    pub gate_obsolete_min_usage: u32,
    /// Failure-rate ceiling for promote gate decision.
    #[serde(default = "memory_defaults::default_gate_promote_failure_rate_ceiling")]
    pub gate_promote_failure_rate_ceiling: f32,
    /// Failure-rate floor for obsolete gate decision.
    #[serde(default = "memory_defaults::default_gate_obsolete_failure_rate_floor")]
    pub gate_obsolete_failure_rate_floor: f32,
    /// Minimum TTL score for promote gate decision.
    #[serde(default = "memory_defaults::default_gate_promote_min_ttl_score")]
    pub gate_promote_min_ttl_score: f32,
    /// Maximum TTL score for obsolete gate decision.
    #[serde(default = "memory_defaults::default_gate_obsolete_max_ttl_score")]
    pub gate_obsolete_max_ttl_score: f32,
    /// Enable Valkey memory stream consumer (`memory.events` -> learning metrics).
    #[serde(default = "memory_defaults::default_stream_consumer_enabled")]
    pub stream_consumer_enabled: bool,
    /// Valkey stream name to consume memory events from.
    #[serde(default = "memory_defaults::default_stream_name")]
    pub stream_name: String,
    /// Consumer group name for memory event stream processing.
    #[serde(default = "memory_defaults::default_stream_consumer_group")]
    pub stream_consumer_group: String,
    /// Consumer name prefix (final consumer name includes pid + timestamp suffix).
    #[serde(default = "memory_defaults::default_stream_consumer_name_prefix")]
    pub stream_consumer_name_prefix: String,
    /// Max events read per XREADGROUP poll.
    #[serde(default = "memory_defaults::default_stream_consumer_batch_size")]
    pub stream_consumer_batch_size: usize,
    /// Block timeout (milliseconds) for XREADGROUP polling.
    #[serde(default = "memory_defaults::default_stream_consumer_block_ms")]
    pub stream_consumer_block_ms: u64,
}

/// Agent config: inference API + external tool server list + optional memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Chat completions endpoint (e.g. `https://api.openai.com/v1/chat/completions` or `LiteLLM`).
    pub inference_url: String,
    /// Model id (e.g. `gpt-4o-mini`, `claude-3-5-sonnet`).
    pub model: String,
    /// API key; if None, read from env `OPENAI_API_KEY` or `ANTHROPIC_API_KEY` depending on URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// External tool servers to connect to (tools from all are merged).
    #[serde(default)]
    pub tool_servers: Vec<ToolServerEntry>,
    /// External tool client-pool size for concurrent tool calls.
    #[serde(default = "agent_defaults::default_tool_pool_size")]
    pub tool_pool_size: usize,
    /// External tool handshake timeout per connect attempt, in seconds.
    #[serde(default = "agent_defaults::default_tool_handshake_timeout_secs")]
    pub tool_handshake_timeout_secs: u64,
    /// External tool connect retries before failing startup.
    #[serde(default = "agent_defaults::default_tool_connect_retries")]
    pub tool_connect_retries: u32,
    /// If true, external tool startup/connect failures abort agent startup.
    /// If false, agent starts without external tools and degrades tool execution gracefully.
    #[serde(default = "agent_defaults::default_tool_strict_startup")]
    pub tool_strict_startup: bool,
    /// Initial backoff between external tool connect retries, in milliseconds.
    #[serde(default = "agent_defaults::default_tool_connect_retry_backoff_ms")]
    pub tool_connect_retry_backoff_ms: u64,
    /// External tool call timeout, in seconds.
    #[serde(default = "agent_defaults::default_tool_timeout_secs")]
    pub tool_timeout_secs: u64,
    /// External tool `tools/list` snapshot cache TTL (milliseconds) on the Rust client side.
    #[serde(default = "agent_defaults::default_tool_list_cache_ttl_ms")]
    pub tool_list_cache_ttl_ms: u64,
    /// Max tool-call rounds per user turn (avoid infinite loops).
    #[serde(default = "agent_defaults::default_max_tool_rounds")]
    pub max_tool_rounds: u32,
    /// Optional xiuxian-memory-engine config (two-phase recall + `store_episode`). None = memory disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemoryConfig>,
    /// If set, use xiuxian-window (ring buffer) for session history with this max turns; context for LLM is built from window. None = use in-memory `SessionStore` (unbounded).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_max_turns: Option<usize>,
    /// When window turn count >= this, consolidate oldest segment into xiuxian-memory-engine. None = consolidation disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consolidation_threshold_turns: Option<usize>,
    /// Number of oldest turns to drain per consolidation (when threshold exceeded). Ignored if consolidation disabled.
    #[serde(default = "agent_defaults::default_consolidation_take_turns")]
    pub consolidation_take_turns: usize,
    /// If true, store consolidated memory episodes in background task.
    #[serde(default = "agent_defaults::default_consolidation_async")]
    pub consolidation_async: bool,
    /// Optional token budget for prompt context packing. None = no token-budget pruning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_budget_tokens: Option<usize>,
    /// Reserved tokens in context budget to avoid packing right at hard limit.
    #[serde(default = "agent_defaults::default_context_budget_reserve_tokens")]
    pub context_budget_reserve_tokens: usize,
    /// Strategy for deciding which context classes are retained first under tight budget.
    #[serde(default)]
    pub context_budget_strategy: ContextBudgetStrategy,
    /// Maximum number of compacted summary segments injected into prompt context.
    #[serde(default = "agent_defaults::default_summary_max_segments")]
    pub summary_max_segments: usize,
    /// Maximum chars kept per compacted summary segment.
    #[serde(default = "agent_defaults::default_summary_max_chars")]
    pub summary_max_chars: usize,
}

/// Prompt context budget retention strategy under tight token constraints.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContextBudgetStrategy {
    /// Keep recent dialogue turns ahead of compacted summary segments.
    #[default]
    RecentFirst,
    /// Keep compacted summary segments ahead of older dialogue turns.
    SummaryFirst,
}

impl ContextBudgetStrategy {
    /// Return canonical `snake_case` label for settings persistence and telemetry.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RecentFirst => "recent_first",
            Self::SummaryFirst => "summary_first",
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            path: memory_defaults::default_memory_path(),
            embedding_backend: None,
            embedding_base_url: None,
            embedding_model: None,
            embedding_batch_max_size: None,
            embedding_batch_max_concurrency: None,
            embedding_timeout_ms: None,
            embedding_timeout_cooldown_ms: None,
            embedding_dim: memory_defaults::default_embedding_dim(),
            table_name: memory_defaults::default_memory_table(),
            recall_k1: memory_defaults::default_recall_k1(),
            recall_k2: memory_defaults::default_recall_k2(),
            recall_lambda: memory_defaults::default_recall_lambda(),
            persistence_backend: memory_defaults::default_memory_persistence_backend(),
            persistence_valkey_url: None,
            persistence_key_prefix: memory_defaults::default_memory_persistence_key_prefix(),
            persistence_strict_startup: None,
            recall_credit_enabled: memory_defaults::default_recall_credit_enabled(),
            recall_credit_max_candidates: memory_defaults::default_recall_credit_max_candidates(),
            decay_enabled: memory_defaults::default_decay_enabled(),
            decay_every_turns: memory_defaults::default_decay_every_turns(),
            decay_factor: memory_defaults::default_decay_factor(),
            gate_promote_threshold: memory_defaults::default_gate_promote_threshold(),
            gate_obsolete_threshold: memory_defaults::default_gate_obsolete_threshold(),
            gate_promote_min_usage: memory_defaults::default_gate_promote_min_usage(),
            gate_obsolete_min_usage: memory_defaults::default_gate_obsolete_min_usage(),
            gate_promote_failure_rate_ceiling:
                memory_defaults::default_gate_promote_failure_rate_ceiling(),
            gate_obsolete_failure_rate_floor:
                memory_defaults::default_gate_obsolete_failure_rate_floor(),
            gate_promote_min_ttl_score: memory_defaults::default_gate_promote_min_ttl_score(),
            gate_obsolete_max_ttl_score: memory_defaults::default_gate_obsolete_max_ttl_score(),
            stream_consumer_enabled: memory_defaults::default_stream_consumer_enabled(),
            stream_name: memory_defaults::default_stream_name(),
            stream_consumer_group: memory_defaults::default_stream_consumer_group(),
            stream_consumer_name_prefix: memory_defaults::default_stream_consumer_name_prefix(),
            stream_consumer_batch_size: memory_defaults::default_stream_consumer_batch_size(),
            stream_consumer_block_ms: memory_defaults::default_stream_consumer_block_ms(),
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            inference_url: "https://api.openai.com/v1/chat/completions".to_string(),
            model: "gpt-4o-mini".to_string(),
            api_key: None,
            tool_servers: Vec::new(),
            tool_pool_size: agent_defaults::default_tool_pool_size(),
            tool_handshake_timeout_secs: agent_defaults::default_tool_handshake_timeout_secs(),
            tool_connect_retries: agent_defaults::default_tool_connect_retries(),
            tool_strict_startup: agent_defaults::default_tool_strict_startup(),
            tool_connect_retry_backoff_ms: agent_defaults::default_tool_connect_retry_backoff_ms(),
            tool_timeout_secs: agent_defaults::default_tool_timeout_secs(),
            tool_list_cache_ttl_ms: agent_defaults::default_tool_list_cache_ttl_ms(),
            max_tool_rounds: agent_defaults::default_max_tool_rounds(),
            memory: None,
            window_max_turns: None,
            consolidation_threshold_turns: None,
            consolidation_take_turns: agent_defaults::default_consolidation_take_turns(),
            consolidation_async: agent_defaults::default_consolidation_async(),
            context_budget_tokens: None,
            context_budget_reserve_tokens: agent_defaults::default_context_budget_reserve_tokens(),
            context_budget_strategy: ContextBudgetStrategy::default(),
            summary_max_segments: agent_defaults::default_summary_max_segments(),
            summary_max_chars: agent_defaults::default_summary_max_chars(),
        }
    }
}

impl AgentConfig {
    /// Build config that uses a `LiteLLM` proxy as the inference endpoint.
    pub fn litellm(model: impl Into<String>) -> Self {
        let inference_url = std::env::var("LITELLM_PROXY_URL")
            .unwrap_or_else(|_| agent_defaults::LITELLM_DEFAULT_URL.to_string());
        let model = std::env::var("XIUXIAN_DAOCHANG_MODEL").unwrap_or_else(|_| model.into());
        Self {
            inference_url,
            model,
            ..Self::default()
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
