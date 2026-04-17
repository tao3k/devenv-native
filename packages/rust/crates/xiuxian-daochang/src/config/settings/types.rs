use std::collections::HashMap;

use serde::Deserialize;

/// Root runtime settings loaded from merged `xiuxian.toml` sources.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RuntimeSettings {
    /// Agent-level runtime settings.
    #[serde(default)]
    pub agent: AgentSettings,
    /// Inference provider settings.
    #[serde(default)]
    pub inference: InferenceSettings,
    /// External tool runtime settings.
    #[serde(default)]
    pub tool_runtime: ToolRuntimeSettings,
    /// Telegram channel runtime settings.
    #[serde(default)]
    pub telegram: TelegramSettings,
    /// Discord channel runtime settings.
    #[serde(default)]
    pub discord: DiscordSettings,
    /// Session storage and context settings.
    #[serde(default)]
    pub session: SessionSettings,
    /// Embedding transport and throughput settings.
    #[serde(default)]
    pub embedding: EmbeddingSettings,
    /// Memory subsystem settings.
    #[serde(default)]
    pub memory: MemorySettings,
    /// Mistral SDK/runtime settings.
    #[serde(default)]
    pub mistral: MistralSettings,
}

/// Agent-level settings section.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AgentSettings {
    /// Optional backend selector (for example `litellm`).
    pub llm_backend: Option<String>,
    /// Agenda validation policy (`always`, `never`, `auto`).
    pub agenda_validation_policy: Option<String>,
}

/// Inference settings section.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct InferenceSettings {
    /// Provider identifier.
    pub provider: Option<String>,
    /// Provider API key value, env key name, or env reference (`env:OPENAI_API_KEY`).
    pub api_key: Option<String>,
    /// Provider base URL.
    pub base_url: Option<String>,
    /// Wire protocol selector (`chat_completions` or `responses`).
    pub wire_api: Option<String>,
    /// Default inference model.
    pub model: Option<String>,
    /// Request timeout in seconds.
    pub timeout: Option<u64>,
    /// Default max tokens for completions.
    pub max_tokens: Option<u64>,
    /// Maximum number of concurrent in-flight inference requests.
    pub max_in_flight: Option<usize>,
}

/// Telegram runtime settings section.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramSettings {
    /// Structured ACL settings.
    #[serde(default)]
    pub acl: TelegramAclSettings,
    /// Persist session admin overrides across restarts.
    pub session_admin_persist: Option<bool>,
    /// Persist `/session partition|scope` mode updates to user settings.
    pub session_partition_persist: Option<bool>,
    /// Group policy mode.
    pub group_policy: Option<String>,
    /// Legacy group allow-from selector list.
    pub group_allow_from: Option<String>,
    /// Require bot mention to process group messages.
    pub require_mention: Option<bool>,
    /// Per-group override map keyed by group id.
    pub groups: Option<HashMap<String, TelegramGroupSettings>>,
    /// Channel mode (`polling`/`webhook`).
    pub mode: Option<String>,
    /// Webhook bind address.
    pub webhook_bind: Option<String>,
    /// Webhook HTTP path.
    pub webhook_path: Option<String>,
    /// Webhook dedup backend selector.
    pub webhook_dedup_backend: Option<String>,
    /// Webhook dedup TTL in seconds.
    pub webhook_dedup_ttl_secs: Option<u64>,
    /// Webhook dedup key prefix.
    pub webhook_dedup_key_prefix: Option<String>,
    /// Maximum tool rounds per turn.
    pub max_tool_rounds: Option<u32>,
    /// Session partition strategy.
    pub session_partition: Option<String>,
    /// Inbound queue capacity.
    pub inbound_queue_capacity: Option<usize>,
    /// Foreground queue capacity.
    pub foreground_queue_capacity: Option<usize>,
    /// Max foreground in-flight messages.
    pub foreground_max_in_flight_messages: Option<usize>,
    /// Foreground turn timeout in seconds.
    pub foreground_turn_timeout_secs: Option<u64>,
    /// Foreground queue mode (`interrupt` or `queue`).
    pub foreground_queue_mode: Option<String>,
    /// Foreground session-gate backend selector.
    pub foreground_session_gate_backend: Option<String>,
    /// Foreground session-gate key prefix.
    pub foreground_session_gate_key_prefix: Option<String>,
    /// Foreground session-gate lease TTL in seconds.
    pub foreground_session_gate_lease_ttl_secs: Option<u64>,
    /// Foreground session-gate acquire timeout in seconds.
    pub foreground_session_gate_acquire_timeout_secs: Option<u64>,
    /// Send-rate gate key prefix.
    pub send_rate_limit_gate_key_prefix: Option<String>,
}

/// Telegram ACL subtree.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramAclSettings {
    /// Base allowlist rules.
    pub allow: Option<TelegramAclAllowSettings>,
    /// Admin principal rules.
    pub admin: Option<TelegramAclPrincipalSettings>,
    /// Control command ACL rules.
    pub control: Option<TelegramAclControlSettings>,
    /// Slash command ACL rules.
    pub slash: Option<TelegramAclSlashSettings>,
}

/// Telegram allowlist principals.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramAclAllowSettings {
    /// Allowed user identifiers.
    pub users: Option<Vec<String>>,
    /// Allowed group identifiers.
    pub groups: Option<Vec<String>>,
}

/// Telegram principal set with user identifiers.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramAclPrincipalSettings {
    /// Allowed user identifiers.
    pub users: Option<Vec<String>>,
}

/// Telegram control command ACL configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramAclControlSettings {
    /// Principal set allowed to run control commands.
    pub allow_from: Option<TelegramAclPrincipalSettings>,
    /// Per-command control ACL rules.
    pub rules: Option<Vec<TelegramAclRuleSettings>>,
}

/// Telegram ACL rule entry for specific command selectors.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramAclRuleSettings {
    /// Command selectors in slash form.
    pub commands: Vec<String>,
    /// Principal set allowed for this rule.
    #[serde(default)]
    pub allow: TelegramAclPrincipalSettings,
}

/// Telegram slash command ACL configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramAclSlashSettings {
    /// Global slash ACL fallback.
    pub global: Option<TelegramAclPrincipalSettings>,
    /// ACL for `/session`.
    pub session_status: Option<TelegramAclPrincipalSettings>,
    /// ACL for `/session budget`.
    pub session_budget: Option<TelegramAclPrincipalSettings>,
    /// ACL for `/session memory`.
    pub session_memory: Option<TelegramAclPrincipalSettings>,
    /// ACL for `/session feedback`.
    pub session_feedback: Option<TelegramAclPrincipalSettings>,
    /// ACL for `/job`.
    pub job_status: Option<TelegramAclPrincipalSettings>,
    /// ACL for `/jobs`.
    pub jobs_summary: Option<TelegramAclPrincipalSettings>,
    /// ACL for background submit command.
    pub background_submit: Option<TelegramAclPrincipalSettings>,
}

/// Telegram per-group override settings.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramGroupSettings {
    /// Whether this group override is enabled.
    pub enabled: Option<bool>,
    /// Group policy mode override.
    pub group_policy: Option<String>,
    /// Control command allow principals for the group.
    pub allow_from: Option<TelegramAclPrincipalSettings>,
    /// Admin principals for the group.
    pub admin_users: Option<TelegramAclPrincipalSettings>,
    /// Require mention override for this group.
    pub require_mention: Option<bool>,
    /// Per-topic overrides keyed by topic id.
    pub topics: Option<HashMap<String, TelegramTopicSettings>>,
}

/// Telegram per-topic override settings.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramTopicSettings {
    /// Whether this topic override is enabled.
    pub enabled: Option<bool>,
    /// Group policy mode override for the topic.
    pub group_policy: Option<String>,
    /// Control command allow principals for the topic.
    pub allow_from: Option<TelegramAclPrincipalSettings>,
    /// Admin principals for the topic.
    pub admin_users: Option<TelegramAclPrincipalSettings>,
    /// Require mention override for this topic.
    pub require_mention: Option<bool>,
}

/// Discord runtime settings section.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DiscordSettings {
    /// Structured ACL settings.
    #[serde(default)]
    pub acl: DiscordAclSettings,
    /// Discord runtime mode selector.
    pub runtime_mode: Option<String>,
    /// Require explicit mention/reply trigger for guild text messages.
    pub require_mention: Option<bool>,
    /// Persist runtime mention-policy mutations into user settings.
    pub require_mention_persist: Option<bool>,
    /// Ingress bind address.
    pub ingress_bind: Option<String>,
    /// Ingress HTTP path.
    pub ingress_path: Option<String>,
    /// Ingress shared secret token.
    pub ingress_secret_token: Option<String>,
    /// Per-channel override map keyed by Discord recipient/channel id or `*`.
    pub channels: Option<HashMap<String, DiscordChannelSettings>>,
    /// Session partition strategy.
    pub session_partition: Option<String>,
    /// Persist `/session partition|scope` mode updates to user settings.
    pub session_partition_persist: Option<bool>,
    /// Inbound queue capacity.
    pub inbound_queue_capacity: Option<usize>,
    /// Per-turn timeout in seconds.
    pub turn_timeout_secs: Option<u64>,
    /// Max foreground in-flight messages.
    pub foreground_max_in_flight_messages: Option<usize>,
    /// Foreground queue mode (`interrupt` or `queue`).
    pub foreground_queue_mode: Option<String>,
}

/// Discord per-channel override settings.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct DiscordChannelSettings {
    /// Require explicit mention in this channel override.
    pub require_mention: Option<bool>,
}

/// Discord ACL subtree.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DiscordAclSettings {
    /// Role alias map from symbolic names to Discord role ids/mentions.
    pub role_aliases: Option<HashMap<String, String>>,
    /// Base allowlist rules.
    pub allow: Option<DiscordAclAllowSettings>,
    /// Admin principal rules.
    pub admin: Option<DiscordAclPrincipalSettings>,
    /// Control command ACL rules.
    pub control: Option<DiscordAclControlSettings>,
    /// Slash command ACL rules.
    pub slash: Option<DiscordAclSlashSettings>,
}

/// Discord allowlist principals.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DiscordAclAllowSettings {
    /// Allowed user identifiers.
    pub users: Option<Vec<String>>,
    /// Allowed role identifiers or aliases.
    pub roles: Option<Vec<String>>,
    /// Allowed guild identifiers.
    pub guilds: Option<Vec<String>>,
}

/// Discord principal settings with user/role selectors.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DiscordAclPrincipalSettings {
    /// Allowed user identifiers.
    pub users: Option<Vec<String>>,
    /// Allowed role identifiers or aliases.
    pub roles: Option<Vec<String>>,
}

/// Discord control command ACL configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DiscordAclControlSettings {
    /// Principal set allowed to run control commands.
    pub allow_from: Option<DiscordAclPrincipalSettings>,
    /// Per-command control ACL rules.
    pub rules: Option<Vec<DiscordAclRuleSettings>>,
}

/// Discord ACL rule entry for command selectors.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DiscordAclRuleSettings {
    /// Command selectors in slash form.
    pub commands: Vec<String>,
    /// Principal set allowed for this rule.
    #[serde(default)]
    pub allow: DiscordAclPrincipalSettings,
}

/// Discord slash command ACL configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DiscordAclSlashSettings {
    /// Global slash ACL fallback.
    pub global: Option<DiscordAclPrincipalSettings>,
    /// ACL for `/session`.
    pub session_status: Option<DiscordAclPrincipalSettings>,
    /// ACL for `/session budget`.
    pub session_budget: Option<DiscordAclPrincipalSettings>,
    /// ACL for `/session memory`.
    pub session_memory: Option<DiscordAclPrincipalSettings>,
    /// ACL for `/session feedback`.
    pub session_feedback: Option<DiscordAclPrincipalSettings>,
    /// ACL for `/job`.
    pub job_status: Option<DiscordAclPrincipalSettings>,
    /// ACL for `/jobs`.
    pub jobs_summary: Option<DiscordAclPrincipalSettings>,
    /// ACL for background submit command.
    pub background_submit: Option<DiscordAclPrincipalSettings>,
}

/// External tool runtime settings section.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ToolRuntimeSettings {
    /// External tool client pool size.
    pub pool_size: Option<usize>,
    /// External tool handshake timeout in seconds.
    pub handshake_timeout_secs: Option<u64>,
    /// Retry attempts for external tool connection establishment.
    pub connect_retries: Option<u32>,
    /// Require strict startup success for external tool endpoints.
    pub strict_startup: Option<bool>,
    /// Backoff between connect retries in milliseconds.
    pub connect_retry_backoff_ms: Option<u64>,
    /// Tool call timeout in seconds.
    pub tool_timeout_secs: Option<u64>,
    /// List-tools cache TTL in milliseconds.
    pub list_tools_cache_ttl_ms: Option<u64>,
    /// Enable discover cache integration.
    pub discover_cache_enabled: Option<bool>,
    /// Discover cache key prefix.
    pub discover_cache_key_prefix: Option<String>,
    /// Discover cache TTL in seconds.
    pub discover_cache_ttl_secs: Option<u64>,
}

/// Session store/runtime settings section.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SessionSettings {
    /// Maximum turns retained in context window.
    pub window_max_turns: Option<usize>,
    /// Turn threshold that triggers consolidation.
    pub consolidation_threshold_turns: Option<usize>,
    /// Number of turns kept after consolidation.
    pub consolidation_take_turns: Option<usize>,
    /// Enable asynchronous consolidation.
    pub consolidation_async: Option<bool>,
    /// Context budget token ceiling.
    pub context_budget_tokens: Option<usize>,
    /// Reserved tokens for response generation.
    pub context_budget_reserve_tokens: Option<usize>,
    /// Context budget strategy identifier.
    pub context_budget_strategy: Option<String>,
    /// Maximum segments in generated summary.
    pub summary_max_segments: Option<usize>,
    /// Maximum summary characters.
    pub summary_max_chars: Option<usize>,
    /// Idle timeout (minutes) before session context is auto-reset.
    pub reset_idle_timeout_mins: Option<u64>,
    /// Maximum characters persisted per chat message content field.
    pub message_content_max_chars: Option<usize>,
    /// Valkey/Redis URL for session backend.
    pub valkey_url: Option<String>,
    /// Redis key prefix.
    pub redis_prefix: Option<String>,
    /// Session TTL in seconds.
    pub ttl_secs: Option<u64>,
}

/// Memory subsystem settings section.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MemorySettings {
    /// Filesystem path for memory store.
    pub path: Option<String>,
    /// Embedding backend selector.
    pub embedding_backend: Option<String>,
    /// Embedding service base URL.
    pub embedding_base_url: Option<String>,
    /// Embedding model identifier.
    pub embedding_model: Option<String>,
    /// Embedding request timeout in milliseconds.
    pub embedding_timeout_ms: Option<u64>,
    /// Cooldown after embedding timeout in milliseconds.
    pub embedding_timeout_cooldown_ms: Option<u64>,
    /// Embedding vector dimension override.
    pub embedding_dim: Option<usize>,
    /// Persistence backend selector.
    pub persistence_backend: Option<String>,
    /// Persistence Valkey URL.
    pub persistence_valkey_url: Option<String>,
    /// Persistence key prefix.
    pub persistence_key_prefix: Option<String>,
    /// Fail startup when persistence backend is unavailable.
    pub persistence_strict_startup: Option<bool>,
    /// Enable recall-credit feedback loop.
    pub recall_credit_enabled: Option<bool>,
    /// Max candidates for recall-credit scoring.
    pub recall_credit_max_candidates: Option<usize>,
    /// Enable periodic decay.
    pub decay_enabled: Option<bool>,
    /// Decay interval measured in turns.
    pub decay_every_turns: Option<usize>,
    /// Decay factor applied per decay interval.
    pub decay_factor: Option<f32>,
    /// Promote threshold for memory gate score.
    pub gate_promote_threshold: Option<f32>,
    /// Obsolete threshold for memory gate score.
    pub gate_obsolete_threshold: Option<f32>,
    /// Minimum usage count required for promote decision.
    pub gate_promote_min_usage: Option<u32>,
    /// Minimum usage count required for obsolete decision.
    pub gate_obsolete_min_usage: Option<u32>,
    /// Max failure-rate allowed for promote decision.
    pub gate_promote_failure_rate_ceiling: Option<f32>,
    /// Min failure-rate required for obsolete decision.
    pub gate_obsolete_failure_rate_floor: Option<f32>,
    /// Minimum TTL score required for promote decision.
    pub gate_promote_min_ttl_score: Option<f32>,
    /// Maximum TTL score allowed for obsolete decision.
    pub gate_obsolete_max_ttl_score: Option<f32>,
    /// Enable stream consumer for external memory events.
    pub stream_consumer_enabled: Option<bool>,
    /// Source stream name for consumer.
    pub stream_name: Option<String>,
    /// Consumer group name.
    pub stream_consumer_group: Option<String>,
    /// Prefix for generated consumer names.
    pub stream_consumer_name_prefix: Option<String>,
    /// Batch size per stream read cycle.
    pub stream_consumer_batch_size: Option<usize>,
    /// Block timeout in milliseconds for stream reads.
    pub stream_consumer_block_ms: Option<u64>,
}

/// Embedding runtime settings section.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EmbeddingSettings {
    /// Embedding backend selector.
    pub backend: Option<String>,
    /// Embedding timeout in seconds.
    #[serde(alias = "timeout")]
    pub timeout_secs: Option<u64>,
    /// Maximum concurrent embedding requests.
    pub max_in_flight: Option<usize>,
    /// Maximum number of items per batch.
    pub batch_max_size: Option<usize>,
    /// Maximum concurrent embedding batches.
    pub batch_max_concurrency: Option<usize>,
    /// Default embedding model.
    pub model: Option<String>,
    /// `LiteLLM` model override.
    pub litellm_model: Option<String>,
    /// `LiteLLM` API base URL.
    pub litellm_api_base: Option<String>,
    /// Embedding dimension override.
    pub dimension: Option<usize>,
    /// Alternate client URL for embedding service.
    pub client_url: Option<String>,
}

/// Mistral runtime settings section.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MistralSettings {
    /// Enable mistral runtime integration.
    pub enabled: Option<bool>,
    /// Auto-start mistral runtime on demand.
    pub auto_start: Option<bool>,
    /// Command used to launch mistral runtime.
    pub command: Option<String>,
    /// Command-line args for mistral runtime command.
    pub args: Option<Vec<String>>,
    /// Mistral runtime base URL.
    pub base_url: Option<String>,
    /// Startup timeout in seconds.
    pub startup_timeout_secs: Option<u64>,
    /// Probe timeout in milliseconds.
    pub probe_timeout_ms: Option<u64>,
    /// Probe interval in milliseconds.
    pub probe_interval_ms: Option<u64>,
    /// HF cache path for mistral SDK artifacts.
    pub sdk_hf_cache_path: Option<String>,
    /// HF revision pin for mistral SDK artifacts.
    pub sdk_hf_revision: Option<String>,
    /// Max concurrent sequences accepted by mistral SDK embedding scheduler.
    pub sdk_embedding_max_num_seqs: Option<usize>,
}
