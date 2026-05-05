//! Agent default values used while constructing runtime config.

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

/// Default `LiteLLM` proxy path (when using `litellm --port 4000`).
pub const LITELLM_DEFAULT_URL: &str = "http://localhost:4000/v1/chat/completions";
