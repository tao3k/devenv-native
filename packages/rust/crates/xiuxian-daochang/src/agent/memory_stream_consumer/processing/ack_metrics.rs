use anyhow::{Context, Result};

use crate::agent::memory_stream_consumer::types::{MemoryStreamConsumerRuntimeConfig, now_unix_ms};

pub(in crate::agent::memory_stream_consumer) async fn ack_and_record_metrics(
    connection: &mut redis::aio::MultiplexedConnection,
    config: &MemoryStreamConsumerRuntimeConfig,
    event_id: &str,
    kind: &str,
    session_id: Option<&str>,
) -> Result<u64> {
    let session_id = session_id.unwrap_or_default();
    let session_metrics_key = format!("{}{}", config.metrics_session_prefix, session_id);
    let ttl_secs = config.ttl_secs.unwrap_or(0);
    let now_unix_ms = now_unix_ms();
    let script = r#"
local stream_key = KEYS[1]
local global_metrics_key = KEYS[2]
local session_metrics_key = KEYS[3]

local consumer_group = ARGV[1]
local event_id = ARGV[2]
local kind = ARGV[3]
local session_id = ARGV[4]
local now_unix_ms = ARGV[5]
local ttl_secs = tonumber(ARGV[6]) or 0
local consumer_name = ARGV[7]

local acked = redis.call("XACK", stream_key, consumer_group, event_id)
if acked > 0 then
  redis.call("HINCRBY", global_metrics_key, "processed_total", acked)
  redis.call("HINCRBY", global_metrics_key, "processed_kind:" .. kind, acked)
  redis.call(
    "HSET",
    global_metrics_key,
    "last_processed_event_id",
    event_id,
    "last_processed_kind",
    kind,
    "last_processed_session_id",
    session_id,
    "last_processed_consumer",
    consumer_name,
    "last_processed_at_unix_ms",
    now_unix_ms
  )
  if session_id ~= "" then
    redis.call("HINCRBY", session_metrics_key, "processed_total", acked)
    redis.call("HINCRBY", session_metrics_key, "processed_kind:" .. kind, acked)
    redis.call(
      "HSET",
      session_metrics_key,
      "last_processed_event_id",
      event_id,
      "last_processed_kind",
      kind,
      "last_processed_consumer",
      consumer_name,
      "last_processed_at_unix_ms",
      now_unix_ms
    )
  end
  if ttl_secs > 0 then
    redis.call("EXPIRE", global_metrics_key, ttl_secs)
    if session_id ~= "" then
      redis.call("EXPIRE", session_metrics_key, ttl_secs)
    end
  end
end
return acked
"#;

    let acked: u64 = redis::cmd("EVAL")
        .arg(script)
        .arg(3)
        .arg(&config.stream_key)
        .arg(&config.metrics_global_key)
        .arg(&session_metrics_key)
        .arg(&config.stream_consumer_group)
        .arg(event_id)
        .arg(kind)
        .arg(session_id)
        .arg(now_unix_ms)
        .arg(ttl_secs)
        .arg(&config.stream_consumer_name)
        .query_async(connection)
        .await
        .context("failed to ack memory stream event and update consumer metrics")?;

    Ok(acked)
}
