use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) const DEFAULT_STREAM_NAME: &str = "memory.events";
pub(super) const DEFAULT_CONSUMER_GROUP: &str = "xiuxian-daochang-memory";
pub(super) const DEFAULT_CONSUMER_PREFIX: &str = "agent";
pub(super) const RECONNECT_BACKOFF_MS: u64 = 500;
const MAX_RECONNECT_BACKOFF_MS: u64 = 30_000;
pub(super) const STREAM_CONSUMER_RESPONSE_TIMEOUT_GRACE_MS: u64 = 500;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::agent) struct MemoryStreamEvent {
    pub(super) id: String,
    pub(super) fields: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub(in crate::agent) struct MemoryStreamConsumerRuntimeConfig {
    pub(super) redis_url: String,
    pub(super) stream_name: String,
    pub(super) stream_key: String,
    pub(super) promotion_stream_key: String,
    pub(super) promotion_ledger_key: String,
    pub(super) stream_consumer_group: String,
    pub(super) stream_consumer_name: String,
    pub(super) stream_consumer_batch_size: usize,
    pub(super) stream_consumer_block_ms: u64,
    pub(super) metrics_global_key: String,
    pub(super) metrics_session_prefix: String,
    pub(super) ttl_secs: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::agent) enum StreamReadErrorKind {
    MissingConsumerGroup,
    Transport,
    Other,
}

pub(in crate::agent) struct StreamEventLogContext<'a> {
    pub(super) event_id: &'a str,
    pub(super) kind: &'a str,
    pub(super) session_id: Option<&'a str>,
}

pub(in crate::agent) fn field_value_or_default(
    fields: &HashMap<String, String>,
    key: &str,
    default: &str,
) -> String {
    fields
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map_or_else(|| default.to_string(), ToString::to_string)
}

pub(in crate::agent::memory_stream_consumer) fn non_empty_string(
    value: Option<String>,
) -> Option<String> {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
}

pub(super) fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
        .unwrap_or(0)
}

pub(super) fn build_consumer_name(prefix: &str) -> String {
    let pid = std::process::id();
    format!("{prefix}-{pid}-{}", now_unix_ms())
}

pub(super) fn compute_retry_backoff_ms(base_ms: u64, failure_streak: u32) -> u64 {
    if failure_streak <= 1 {
        return base_ms.max(1);
    }
    let shift = failure_streak.saturating_sub(1).min(12);
    base_ms
        .max(1)
        .saturating_mul(1u64 << shift)
        .min(MAX_RECONNECT_BACKOFF_MS)
}

pub(in crate::agent) fn bump_failure_streak_and_backoff(failure_streak: &mut u32) -> u64 {
    *failure_streak = failure_streak.saturating_add(1);
    compute_retry_backoff_ms(RECONNECT_BACKOFF_MS, *failure_streak)
}

pub(super) fn reconnect_backoff_ms(failure_streak: u32) -> u64 {
    compute_retry_backoff_ms(RECONNECT_BACKOFF_MS, failure_streak.max(1))
}
