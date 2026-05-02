//! Memory stream consumer branch for valkey stream ingestion.

mod bootstrap;
mod parsing;
mod processing;
mod runtime;
mod stream;
mod test_api;
mod types;

pub(super) use bootstrap::spawn_memory_stream_consumer;
pub(crate) use test_api::{
    TestMemoryStreamConsumerRuntimeConfig, TestMemoryStreamEvent, TestStreamReadErrorKind,
    test_ack_and_record_metrics, test_build_consumer_name, test_classify_stream_read_error,
    test_compute_retry_backoff_ms, test_ensure_consumer_group, test_is_idle_poll_timeout_error,
    test_parse_xreadgroup_reply, test_queue_promoted_candidate, test_read_stream_events,
    test_stream_consumer_connection_config, test_stream_consumer_response_timeout,
    test_summarize_redis_error,
};
pub(super) use types::{MemoryStreamConsumerRuntimeConfig, MemoryStreamEvent, StreamReadErrorKind};
