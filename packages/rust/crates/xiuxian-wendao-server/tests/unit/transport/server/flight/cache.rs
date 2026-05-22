use std::sync::Arc;

use arrow_array::RecordBatch;
use arrow_schema::Schema;

use super::cache::FlightRoutePayloadCache;
use super::payload::FlightRoutePayload;

#[tokio::test]
async fn handoff_is_one_shot() {
    let cache = FlightRoutePayloadCache::default();
    let payload = Arc::new(test_payload());

    cache
        .insert_handoff("search-key".to_string(), Arc::clone(&payload))
        .await;

    let first = cache.take_handoff("search-key").await;
    let second = cache.take_handoff("search-key").await;

    assert!(first.is_some());
    assert!(second.is_none());
}

#[tokio::test]
async fn payload_alias_overwrites_existing_cache_entry() {
    let cache = FlightRoutePayloadCache::default();
    let stale = cache
        .insert("document-key".to_string(), test_payload())
        .await;
    let fresh = Arc::new(test_payload());

    cache
        .insert_alias("document-key".to_string(), Arc::clone(&fresh))
        .await;
    let cached = cache
        .get("document-key")
        .await
        .unwrap_or_else(|| panic!("alias should be cached"));

    assert!(!Arc::ptr_eq(&cached, &stale));
    assert!(Arc::ptr_eq(&cached, &fresh));
}

fn test_payload() -> FlightRoutePayload {
    let schema = Arc::new(Schema::empty());
    let batch = RecordBatch::new_empty(schema);
    FlightRoutePayload::from_batches_with_app_metadata(&[batch], Vec::new())
        .unwrap_or_else(|error| panic!("test payload: {error}"))
}
