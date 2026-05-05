use crate::agent::logging::should_surface_repeated_failure;
use crate::observability::SessionEvent;

use crate::agent::memory_stream_consumer::types::MemoryStreamConsumerRuntimeConfig;

pub(super) fn log_missing_consumer_group_recovery_attempt(
    config: &MemoryStreamConsumerRuntimeConfig,
    stream_id: &str,
    error: &anyhow::Error,
    retry_backoff_ms: u64,
    failure_streak: u32,
) {
    if should_surface_repeated_failure(failure_streak) {
        tracing::info!(
            event = SessionEvent::MemoryStreamConsumerReadFailed.as_str(),
            stream_name = %config.stream_name,
            stream_consumer_group = %config.stream_consumer_group,
            stream_consumer_name = %config.stream_consumer_name,
            stream_id,
            error_kind = "missing_consumer_group",
            failure_streak,
            retry_backoff_ms,
            error = %error,
            "memory stream consumer detected missing consumer group; attempting recovery"
        );
    } else {
        tracing::trace!(
            event = SessionEvent::MemoryStreamConsumerReadFailed.as_str(),
            stream_name = %config.stream_name,
            stream_consumer_group = %config.stream_consumer_group,
            stream_consumer_name = %config.stream_consumer_name,
            stream_id,
            error_kind = "missing_consumer_group",
            failure_streak,
            retry_backoff_ms,
            error = %error,
            "memory stream consumer detected missing consumer group; attempting recovery"
        );
    }
}

pub(super) fn log_missing_consumer_group_recovery_failure(
    config: &MemoryStreamConsumerRuntimeConfig,
    stream_id: &str,
    error: &anyhow::Error,
    ensure_error: &anyhow::Error,
    retry_backoff_ms: u64,
    failure_streak: u32,
) {
    if should_surface_repeated_failure(failure_streak) {
        tracing::warn!(
            event = SessionEvent::MemoryStreamConsumerReadFailed.as_str(),
            stream_name = %config.stream_name,
            stream_consumer_group = %config.stream_consumer_group,
            stream_consumer_name = %config.stream_consumer_name,
            stream_id,
            error_kind = "missing_consumer_group_recovery_failed",
            failure_streak,
            retry_backoff_ms,
            error = %error,
            ensure_error = %ensure_error,
            "memory stream consumer group recovery failed; reconnecting"
        );
    } else {
        tracing::trace!(
            event = SessionEvent::MemoryStreamConsumerReadFailed.as_str(),
            stream_name = %config.stream_name,
            stream_consumer_group = %config.stream_consumer_group,
            stream_consumer_name = %config.stream_consumer_name,
            stream_id,
            error_kind = "missing_consumer_group_recovery_failed",
            failure_streak,
            retry_backoff_ms,
            error = %error,
            ensure_error = %ensure_error,
            "memory stream consumer group recovery failed; reconnecting"
        );
    }
}

pub(super) fn log_stream_read_reconnect(
    config: &MemoryStreamConsumerRuntimeConfig,
    stream_id: &str,
    error_kind: &str,
    error: &anyhow::Error,
    retry_backoff_ms: u64,
    failure_streak: u32,
) {
    if should_surface_repeated_failure(failure_streak) {
        tracing::warn!(
            event = SessionEvent::MemoryStreamConsumerReadFailed.as_str(),
            stream_name = %config.stream_name,
            stream_consumer_group = %config.stream_consumer_group,
            stream_consumer_name = %config.stream_consumer_name,
            stream_id,
            error_kind,
            failure_streak,
            retry_backoff_ms,
            error = %error,
            "memory stream consumer read failed; reconnecting"
        );
    } else {
        tracing::trace!(
            event = SessionEvent::MemoryStreamConsumerReadFailed.as_str(),
            stream_name = %config.stream_name,
            stream_consumer_group = %config.stream_consumer_group,
            stream_consumer_name = %config.stream_consumer_name,
            stream_id,
            error_kind,
            failure_streak,
            retry_backoff_ms,
            error = %error,
            "memory stream consumer read failed; reconnecting"
        );
    }
}
