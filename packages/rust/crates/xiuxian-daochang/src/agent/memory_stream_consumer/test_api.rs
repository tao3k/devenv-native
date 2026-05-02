//! Test adapter API for memory stream consumer internals.

use std::collections::HashMap;
use std::time::Duration;

use super::{MemoryStreamConsumerRuntimeConfig, MemoryStreamEvent, StreamReadErrorKind};
use super::{parsing, processing, runtime, stream};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TestMemoryStreamEvent {
    pub id: String,
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TestMemoryStreamConsumerRuntimeConfig {
    pub redis_url: String,
    pub stream_name: String,
    pub stream_key: String,
    pub promotion_stream_key: String,
    pub promotion_ledger_key: String,
    pub stream_consumer_group: String,
    pub stream_consumer_name: String,
    pub stream_consumer_batch_size: usize,
    pub stream_consumer_block_ms: u64,
    pub metrics_global_key: String,
    pub metrics_session_prefix: String,
    pub ttl_secs: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TestStreamReadErrorKind {
    MissingConsumerGroup,
    Transport,
    Other,
}

pub(crate) fn test_parse_xreadgroup_reply(
    reply: redis::Value,
) -> anyhow::Result<Vec<TestMemoryStreamEvent>> {
    parsing::parse_xreadgroup_reply(reply).map(|events| {
        events
            .into_iter()
            .map(from_internal_event)
            .collect::<Vec<_>>()
    })
}

pub(crate) fn test_build_consumer_name(prefix: &str) -> String {
    super::types::build_consumer_name(prefix)
}

pub(crate) fn test_compute_retry_backoff_ms(base_ms: u64, failure_streak: u32) -> u64 {
    super::types::compute_retry_backoff_ms(base_ms, failure_streak)
}

pub(crate) fn test_classify_stream_read_error(error: &anyhow::Error) -> TestStreamReadErrorKind {
    from_internal_stream_read_error_kind(runtime::read_error::classify_stream_read_error(error))
}

pub(crate) fn test_stream_consumer_response_timeout(block_ms: u64) -> Duration {
    stream::stream_consumer_response_timeout(block_ms)
}

pub(crate) fn test_stream_consumer_connection_config(
    block_ms: u64,
) -> redis::AsyncConnectionConfig {
    stream::stream_consumer_connection_config(block_ms)
}

pub(crate) fn test_summarize_redis_error(error: &redis::RedisError) -> String {
    stream::summarize_redis_error(error)
}

pub(crate) fn test_is_idle_poll_timeout_error(error: &redis::RedisError) -> bool {
    stream::is_idle_poll_timeout_error(error)
}

pub(crate) async fn test_ensure_consumer_group(
    connection: &mut redis::aio::MultiplexedConnection,
    config: &TestMemoryStreamConsumerRuntimeConfig,
) -> anyhow::Result<()> {
    let internal = to_internal_config(config);
    stream::ensure_consumer_group(connection, &internal).await
}

pub(crate) async fn test_read_stream_events(
    connection: &mut redis::aio::MultiplexedConnection,
    config: &TestMemoryStreamConsumerRuntimeConfig,
    stream_id: &str,
) -> anyhow::Result<Vec<TestMemoryStreamEvent>> {
    let internal = to_internal_config(config);
    stream::read_stream_events(connection, &internal, stream_id)
        .await
        .map(|events| {
            events
                .into_iter()
                .map(from_internal_event)
                .collect::<Vec<_>>()
        })
}

pub(crate) async fn test_ack_and_record_metrics(
    connection: &mut redis::aio::MultiplexedConnection,
    config: &TestMemoryStreamConsumerRuntimeConfig,
    event_id: &str,
    kind: &str,
    session_id: Option<&str>,
) -> anyhow::Result<u64> {
    let internal = to_internal_config(config);
    processing::ack_and_record_metrics(connection, &internal, event_id, kind, session_id).await
}

pub(crate) async fn test_queue_promoted_candidate(
    connection: &mut redis::aio::MultiplexedConnection,
    config: &TestMemoryStreamConsumerRuntimeConfig,
    event: &TestMemoryStreamEvent,
) -> anyhow::Result<bool> {
    let internal = to_internal_config(config);
    let internal_event = to_internal_event(event);
    processing::queue_promoted_candidate(connection, &internal, &internal_event).await
}

fn to_internal_config(
    config: &TestMemoryStreamConsumerRuntimeConfig,
) -> MemoryStreamConsumerRuntimeConfig {
    MemoryStreamConsumerRuntimeConfig {
        redis_url: config.redis_url.clone(),
        stream_name: config.stream_name.clone(),
        stream_key: config.stream_key.clone(),
        promotion_stream_key: config.promotion_stream_key.clone(),
        promotion_ledger_key: config.promotion_ledger_key.clone(),
        stream_consumer_group: config.stream_consumer_group.clone(),
        stream_consumer_name: config.stream_consumer_name.clone(),
        stream_consumer_batch_size: config.stream_consumer_batch_size,
        stream_consumer_block_ms: config.stream_consumer_block_ms,
        metrics_global_key: config.metrics_global_key.clone(),
        metrics_session_prefix: config.metrics_session_prefix.clone(),
        ttl_secs: config.ttl_secs,
    }
}

fn from_internal_event(event: MemoryStreamEvent) -> TestMemoryStreamEvent {
    TestMemoryStreamEvent {
        id: event.id,
        fields: event.fields,
    }
}

fn to_internal_event(event: &TestMemoryStreamEvent) -> MemoryStreamEvent {
    MemoryStreamEvent {
        id: event.id.clone(),
        fields: event.fields.clone(),
    }
}

const fn from_internal_stream_read_error_kind(
    kind: StreamReadErrorKind,
) -> TestStreamReadErrorKind {
    match kind {
        StreamReadErrorKind::MissingConsumerGroup => TestStreamReadErrorKind::MissingConsumerGroup,
        StreamReadErrorKind::Transport => TestStreamReadErrorKind::Transport,
        StreamReadErrorKind::Other => TestStreamReadErrorKind::Other,
    }
}
