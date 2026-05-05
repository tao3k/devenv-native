use anyhow::Result;
use xiuxian_daochang::test_support::{
    ack_and_record_metrics, ensure_consumer_group, queue_promoted_candidate, read_stream_events,
};

use super::support::{
    append_promoted_memory_event, assert_single_promoted_queue_entry,
    build_promoted_queue_test_context, cleanup_promoted_queue_test_keys, live_redis_url,
};

#[tokio::test]
async fn memory_promoted_events_are_queued_once_for_knowledge_ingest() -> Result<()> {
    let Some(redis_url) = live_redis_url() else {
        return Ok(());
    };

    let context = build_promoted_queue_test_context(&redis_url);

    let client = redis::Client::open(redis_url.as_str())?;
    let mut connection = client.get_multiplexed_async_connection().await?;
    ensure_consumer_group(&mut connection, &context.config).await?;

    let event_id =
        append_promoted_memory_event(&mut connection, &context.config.stream_key).await?;
    let events = read_stream_events(&mut connection, &context.config, ">").await?;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, event_id);

    let inserted = queue_promoted_candidate(&mut connection, &context.config, &events[0]).await?;
    assert!(inserted, "first promoted event should be inserted");
    let inserted_again =
        queue_promoted_candidate(&mut connection, &context.config, &events[0]).await?;
    assert!(
        !inserted_again,
        "duplicate promoted event should be deduplicated"
    );

    let acked = ack_and_record_metrics(
        &mut connection,
        &context.config,
        &events[0].id,
        events[0]
            .fields
            .get("kind")
            .map_or("unknown", String::as_str),
        events[0].fields.get("session_id").map(String::as_str),
    )
    .await?;
    assert_eq!(acked, 1);

    assert_single_promoted_queue_entry(&mut connection, &context.config).await?;
    cleanup_promoted_queue_test_keys(&mut connection, &context).await?;

    Ok(())
}
