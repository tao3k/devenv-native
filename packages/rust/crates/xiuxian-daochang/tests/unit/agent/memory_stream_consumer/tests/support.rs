use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use redis::Value;
use xiuxian_daochang::test_support::{
    MemoryStreamConsumerRuntimeConfig, build_consumer_name, parse_xreadgroup_reply,
};

use crate::unit::live_gates::{live_valkey_enabled, resolve_live_valkey_url};

pub(super) const PROMOTED_SESSION_ID: &str = "telegram:test:promoted";
pub(super) const PROMOTED_EPISODE_ID: &str = "turn-telegram:test:promoted-1";
pub(super) const PROMOTION_HINT: &str = "knowledge.ingest_candidate";

pub(super) struct PromotedQueueTestContext {
    pub(super) config: MemoryStreamConsumerRuntimeConfig,
    pub(super) session_metrics_key: String,
}

pub(super) fn unique_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{prefix}-{nanos}")
}

pub(super) fn live_redis_url() -> Option<String> {
    if !live_valkey_enabled() {
        return None;
    }
    resolve_live_valkey_url()
}

pub(super) fn build_promoted_queue_test_context(redis_url: &str) -> PromotedQueueTestContext {
    let key_prefix = unique_id("xiuxian-daochang-memory-promoted-queue");
    let stream_name = "memory.events";
    let metrics_session_prefix = format!("{key_prefix}:metrics:{stream_name}:consumer:session:");
    let config = MemoryStreamConsumerRuntimeConfig {
        redis_url: redis_url.to_string(),
        stream_name: stream_name.to_string(),
        stream_key: format!("{key_prefix}:stream:{stream_name}"),
        promotion_stream_key: format!("{key_prefix}:stream:knowledge.ingest.candidates"),
        promotion_ledger_key: format!("{key_prefix}:knowledge:ingest:candidates"),
        stream_consumer_group: "xiuxian-daochang-memory-test".to_string(),
        stream_consumer_name: build_consumer_name("agent-test"),
        stream_consumer_batch_size: 16,
        stream_consumer_block_ms: 100,
        metrics_global_key: format!("{key_prefix}:metrics:{stream_name}:consumer"),
        metrics_session_prefix: metrics_session_prefix.clone(),
        ttl_secs: Some(120),
    };
    let session_metrics_key = format!("{metrics_session_prefix}{PROMOTED_SESSION_ID}");
    PromotedQueueTestContext {
        config,
        session_metrics_key,
    }
}

pub(super) async fn append_promoted_memory_event(
    connection: &mut redis::aio::MultiplexedConnection,
    stream_key: &str,
) -> Result<String> {
    redis::cmd("XADD")
        .arg(stream_key)
        .arg("*")
        .arg("kind")
        .arg("memory_promoted")
        .arg("session_id")
        .arg(PROMOTED_SESSION_ID)
        .arg("episode_id")
        .arg(PROMOTED_EPISODE_ID)
        .arg("utility_score")
        .arg("0.93")
        .arg("ttl_score")
        .arg("0.84")
        .arg("knowledge_ingest_hint")
        .arg(PROMOTION_HINT)
        .query_async(connection)
        .await
        .map_err(Into::into)
}

pub(super) async fn assert_single_promoted_queue_entry(
    connection: &mut redis::aio::MultiplexedConnection,
    config: &MemoryStreamConsumerRuntimeConfig,
) -> Result<()> {
    let queued_count: usize = redis::cmd("XLEN")
        .arg(&config.promotion_stream_key)
        .query_async(connection)
        .await?;
    assert_eq!(queued_count, 1, "promoted event should queue exactly once");

    let ledger_payload: Option<String> = redis::cmd("HGET")
        .arg(&config.promotion_ledger_key)
        .arg(PROMOTED_EPISODE_ID)
        .query_async(connection)
        .await?;
    let Some(ledger_payload) = ledger_payload else {
        panic!("expected promotion ledger payload");
    };
    assert!(
        ledger_payload.contains("\"kind\":\"memory_promoted\""),
        "ledger payload should include source event kind"
    );
    Ok(())
}

pub(super) async fn cleanup_promoted_queue_test_keys(
    connection: &mut redis::aio::MultiplexedConnection,
    context: &PromotedQueueTestContext,
) -> Result<()> {
    let _: () = redis::pipe()
        .cmd("DEL")
        .arg(&context.config.stream_key)
        .ignore()
        .cmd("DEL")
        .arg(&context.config.metrics_global_key)
        .ignore()
        .cmd("DEL")
        .arg(&context.session_metrics_key)
        .ignore()
        .cmd("DEL")
        .arg(&context.config.promotion_stream_key)
        .ignore()
        .cmd("DEL")
        .arg(&context.config.promotion_ledger_key)
        .ignore()
        .query_async(connection)
        .await?;
    Ok(())
}

pub(super) fn parse_array_reply() -> Result<Vec<xiuxian_daochang::test_support::MemoryStreamEvent>>
{
    let reply = Value::Array(vec![Value::Array(vec![
        Value::BulkString(b"xiuxian-daochang:stream:memory.events".to_vec()),
        Value::Array(vec![
            Value::Array(vec![
                Value::BulkString(b"1740000000000-0".to_vec()),
                Value::Array(vec![
                    Value::BulkString(b"kind".to_vec()),
                    Value::BulkString(b"turn_stored".to_vec()),
                    Value::BulkString(b"session_id".to_vec()),
                    Value::BulkString(b"telegram:1:1".to_vec()),
                ]),
            ]),
            Value::Array(vec![
                Value::BulkString(b"1740000000001-0".to_vec()),
                Value::Array(vec![
                    Value::BulkString(b"kind".to_vec()),
                    Value::BulkString(b"consolidation_stored".to_vec()),
                ]),
            ]),
        ]),
    ])]);
    parse_xreadgroup_reply(reply)
}

pub(super) fn parse_map_reply() -> Result<Vec<xiuxian_daochang::test_support::MemoryStreamEvent>> {
    let reply = Value::Map(vec![(
        Value::BulkString(b"xiuxian-daochang:stream:memory.events".to_vec()),
        Value::Array(vec![Value::Array(vec![
            Value::BulkString(b"1740000001000-0".to_vec()),
            Value::Map(vec![
                (
                    Value::BulkString(b"kind".to_vec()),
                    Value::BulkString(b"recall_snapshot_updated".to_vec()),
                ),
                (
                    Value::BulkString(b"session_id".to_vec()),
                    Value::BulkString(b"telegram:9:9".to_vec()),
                ),
            ]),
        ])]),
    )]);
    parse_xreadgroup_reply(reply)
}
