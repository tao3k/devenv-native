use anyhow::{Context, Result, bail};
use serde_json::Value as JsonValue;

use crate::agent::memory_stream_consumer::types::{
    MemoryStreamConsumerRuntimeConfig, MemoryStreamEvent, field_value_or_default, now_unix_ms,
};

pub(in crate::agent::memory_stream_consumer) async fn queue_promoted_candidate(
    connection: &mut redis::aio::MultiplexedConnection,
    config: &MemoryStreamConsumerRuntimeConfig,
    event: &MemoryStreamEvent,
) -> Result<bool> {
    let episode_id = field_value_or_default(&event.fields, "episode_id", &event.id);
    if episode_id.is_empty() {
        bail!("memory_promoted event missing episode_id");
    }
    let session_id = field_value_or_default(&event.fields, "session_id", "");
    let payload = serialize_promoted_payload(event)?;
    let now_unix_ms = now_unix_ms();
    let script = r#"
local ledger_key = KEYS[1]
local ingest_stream_key = KEYS[2]
local episode_id = ARGV[1]
local payload = ARGV[2]
local source_event_id = ARGV[3]
local session_id = ARGV[4]
local now_unix_ms = ARGV[5]

local inserted = redis.call("HSETNX", ledger_key, episode_id, payload)
if inserted > 0 then
  redis.call(
    "XADD",
    ingest_stream_key,
    "*",
    "kind",
    "knowledge_ingest_candidate",
    "source_kind",
    "memory_promoted",
    "episode_id",
    episode_id,
    "session_id",
    session_id,
    "source_event_id",
    source_event_id,
    "created_at_unix_ms",
    now_unix_ms,
    "payload",
    payload
  )
end
return inserted
"#;
    let inserted: i64 = redis::cmd("EVAL")
        .arg(script)
        .arg(2)
        .arg(&config.promotion_ledger_key)
        .arg(&config.promotion_stream_key)
        .arg(episode_id)
        .arg(payload)
        .arg(&event.id)
        .arg(session_id)
        .arg(now_unix_ms)
        .query_async(connection)
        .await
        .context("failed to queue memory_promoted event into knowledge ingest stream")?;

    Ok(inserted > 0)
}

fn serialize_promoted_payload(event: &MemoryStreamEvent) -> Result<String> {
    let mut payload = serde_json::Map::new();
    payload.insert(
        "stream_event_id".to_string(),
        JsonValue::String(event.id.clone()),
    );
    for (key, value) in &event.fields {
        payload.insert(key.clone(), JsonValue::String(value.clone()));
    }
    serde_json::to_string(&payload).context("failed to serialize promoted memory payload")
}
