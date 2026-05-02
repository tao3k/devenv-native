use std::collections::HashMap;

use anyhow::Result;
use tokio::time::{Duration, sleep};
use xiuxian_daochang::test_support::{
    MemoryStreamConsumerRuntimeConfig, StreamReadErrorKind, ack_and_record_metrics,
    build_consumer_name, classify_stream_read_error, ensure_consumer_group, read_stream_events,
    stream_consumer_connection_config,
};

use super::support::{live_redis_url, unique_id};

#[tokio::test]
async fn memory_stream_consumer_acks_and_tracks_metrics() -> Result<()> {
    let Some(redis_url) = live_redis_url() else {
        return Ok(());
    };

    let key_prefix = unique_id("xiuxian-daochang-memory-stream-consumer");
    let stream_name = "memory.events".to_string();
    let stream_key = format!("{key_prefix}:stream:{stream_name}");
    let stream_consumer_group = "xiuxian-daochang-memory-test".to_string();
    let stream_consumer_name = build_consumer_name("agent-test");
    let metrics_global_key = format!("{key_prefix}:metrics:{stream_name}:consumer");
    let metrics_session_prefix = format!("{key_prefix}:metrics:{stream_name}:consumer:session:");
    let config = MemoryStreamConsumerRuntimeConfig {
        redis_url: redis_url.clone(),
        stream_name: stream_name.clone(),
        stream_key: stream_key.clone(),
        promotion_stream_key: format!("{key_prefix}:stream:knowledge.ingest.candidates"),
        promotion_ledger_key: format!("{key_prefix}:knowledge:ingest:candidates"),
        stream_consumer_group: stream_consumer_group.clone(),
        stream_consumer_name: stream_consumer_name.clone(),
        stream_consumer_batch_size: 16,
        stream_consumer_block_ms: 100,
        metrics_global_key: metrics_global_key.clone(),
        metrics_session_prefix: metrics_session_prefix.clone(),
        ttl_secs: Some(120),
    };

    let client = redis::Client::open(redis_url.as_str())?;
    let mut connection = client.get_multiplexed_async_connection().await?;
    ensure_consumer_group(&mut connection, &config).await?;

    let event_id: String = redis::cmd("XADD")
        .arg(&stream_key)
        .arg("*")
        .arg("kind")
        .arg("turn_stored")
        .arg("session_id")
        .arg("telegram:test:1")
        .query_async(&mut connection)
        .await?;

    let events = read_stream_events(&mut connection, &config, ">").await?;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, event_id);

    let acked = ack_and_record_metrics(
        &mut connection,
        &config,
        &events[0].id,
        events[0]
            .fields
            .get("kind")
            .map_or("unknown", String::as_str),
        events[0].fields.get("session_id").map(String::as_str),
    )
    .await?;
    assert_eq!(acked, 1);

    let duplicate_ack: u64 = redis::cmd("XACK")
        .arg(&stream_key)
        .arg(&stream_consumer_group)
        .arg(&event_id)
        .query_async(&mut connection)
        .await?;
    assert_eq!(duplicate_ack, 0);

    let global_metrics: HashMap<String, String> = redis::cmd("HGETALL")
        .arg(&metrics_global_key)
        .query_async(&mut connection)
        .await?;
    assert_eq!(
        global_metrics.get("processed_total").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        global_metrics
            .get("processed_kind:turn_stored")
            .map(String::as_str),
        Some("1")
    );

    let session_metrics_key = format!("{metrics_session_prefix}telegram:test:1");
    let session_metrics: HashMap<String, String> = redis::cmd("HGETALL")
        .arg(&session_metrics_key)
        .query_async(&mut connection)
        .await?;
    assert_eq!(
        session_metrics.get("processed_total").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        session_metrics
            .get("processed_kind:turn_stored")
            .map(String::as_str),
        Some("1")
    );

    let _: () = redis::pipe()
        .cmd("DEL")
        .arg(&stream_key)
        .ignore()
        .cmd("DEL")
        .arg(&metrics_global_key)
        .ignore()
        .cmd("DEL")
        .arg(&session_metrics_key)
        .ignore()
        .query_async(&mut connection)
        .await?;

    Ok(())
}

#[tokio::test]
async fn memory_stream_consumer_read_empty_stream_returns_empty() -> Result<()> {
    let Some(redis_url) = live_redis_url() else {
        return Ok(());
    };

    let key_prefix = unique_id("xiuxian-daochang-memory-stream-empty");
    let stream_name = "memory.events".to_string();
    let stream_key = format!("{key_prefix}:stream:{stream_name}");
    let stream_consumer_group = "xiuxian-daochang-memory-test".to_string();
    let stream_consumer_name = build_consumer_name("agent-test");
    let config = MemoryStreamConsumerRuntimeConfig {
        redis_url: redis_url.clone(),
        stream_name: stream_name.clone(),
        stream_key: stream_key.clone(),
        promotion_stream_key: format!("{key_prefix}:stream:knowledge.ingest.candidates"),
        promotion_ledger_key: format!("{key_prefix}:knowledge:ingest:candidates"),
        stream_consumer_group: stream_consumer_group.clone(),
        stream_consumer_name,
        stream_consumer_batch_size: 8,
        stream_consumer_block_ms: 1_000,
        metrics_global_key: format!("{key_prefix}:metrics:{stream_name}:consumer"),
        metrics_session_prefix: format!("{key_prefix}:metrics:{stream_name}:consumer:session:"),
        ttl_secs: Some(120),
    };

    let client = redis::Client::open(redis_url.as_str())?;
    let connection_config = stream_consumer_connection_config(config.stream_consumer_block_ms);
    let mut connection = client
        .get_multiplexed_async_connection_with_config(&connection_config)
        .await?;
    ensure_consumer_group(&mut connection, &config).await?;

    let events = read_stream_events(&mut connection, &config, ">").await?;
    assert!(events.is_empty(), "expected empty read from idle stream");

    let _: () = redis::cmd("DEL")
        .arg(&stream_key)
        .query_async(&mut connection)
        .await?;

    Ok(())
}

#[tokio::test]
async fn memory_stream_consumer_recovers_after_stream_key_expired() -> Result<()> {
    let Some(redis_url) = live_redis_url() else {
        return Ok(());
    };

    let key_prefix = unique_id("xiuxian-daochang-memory-stream-expired");
    let stream_name = "memory.events".to_string();
    let stream_key = format!("{key_prefix}:stream:{stream_name}");
    let stream_consumer_group = "xiuxian-daochang-memory-test".to_string();
    let stream_consumer_name = build_consumer_name("agent-test");
    let metrics_global_key = format!("{key_prefix}:metrics:{stream_name}:consumer");
    let metrics_session_prefix = format!("{key_prefix}:metrics:{stream_name}:consumer:session:");
    let config = MemoryStreamConsumerRuntimeConfig {
        redis_url: redis_url.clone(),
        stream_name: stream_name.clone(),
        stream_key: stream_key.clone(),
        promotion_stream_key: format!("{key_prefix}:stream:knowledge.ingest.candidates"),
        promotion_ledger_key: format!("{key_prefix}:knowledge:ingest:candidates"),
        stream_consumer_group: stream_consumer_group.clone(),
        stream_consumer_name: stream_consumer_name.clone(),
        stream_consumer_batch_size: 16,
        stream_consumer_block_ms: 50,
        metrics_global_key: metrics_global_key.clone(),
        metrics_session_prefix: metrics_session_prefix.clone(),
        ttl_secs: Some(120),
    };

    let client = redis::Client::open(redis_url.as_str())?;
    let mut connection = client.get_multiplexed_async_connection().await?;

    ensure_consumer_group(&mut connection, &config).await?;

    let first_event_id: String = redis::cmd("XADD")
        .arg(&stream_key)
        .arg("*")
        .arg("kind")
        .arg("turn_stored")
        .arg("session_id")
        .arg("telegram:test:expire")
        .query_async(&mut connection)
        .await?;

    let first_events = read_stream_events(&mut connection, &config, ">").await?;
    assert_eq!(first_events.len(), 1);
    assert_eq!(first_events[0].id, first_event_id);

    let _: bool = redis::cmd("EXPIRE")
        .arg(&stream_key)
        .arg(1)
        .query_async(&mut connection)
        .await?;
    sleep(Duration::from_millis(1_200)).await;

    let exists_after_expire: i64 = redis::cmd("EXISTS")
        .arg(&stream_key)
        .query_async(&mut connection)
        .await?;
    assert_eq!(exists_after_expire, 0);

    let Err(expired_read_error) = read_stream_events(&mut connection, &config, ">").await else {
        panic!("expected NOGROUP after stream key expiration");
    };
    assert_eq!(
        classify_stream_read_error(&expired_read_error),
        StreamReadErrorKind::MissingConsumerGroup
    );

    ensure_consumer_group(&mut connection, &config).await?;

    let recovered_event_id: String = redis::cmd("XADD")
        .arg(&stream_key)
        .arg("*")
        .arg("kind")
        .arg("turn_stored")
        .arg("session_id")
        .arg("telegram:test:expire")
        .query_async(&mut connection)
        .await?;
    let recovered_events = read_stream_events(&mut connection, &config, ">").await?;
    assert_eq!(recovered_events.len(), 1);
    assert_eq!(recovered_events[0].id, recovered_event_id);

    let _: () = redis::pipe()
        .cmd("DEL")
        .arg(&stream_key)
        .ignore()
        .cmd("DEL")
        .arg(&metrics_global_key)
        .ignore()
        .query_async(&mut connection)
        .await?;

    Ok(())
}
