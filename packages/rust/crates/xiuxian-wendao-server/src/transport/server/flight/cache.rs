use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use super::payload::FlightRoutePayload;

const MAX_CACHED_ROUTE_PAYLOADS: usize = 128;
const MAX_ROUTE_PAYLOAD_HANDOFFS: usize = 128;
const ROUTE_PAYLOAD_HANDOFF_TTL: Duration = Duration::from_secs(1);

#[derive(Debug)]
struct FlightRoutePayloadHandoff {
    payload: Arc<FlightRoutePayload>,
    expires_at: Instant,
}

#[derive(Debug, Default)]
pub(super) struct FlightRoutePayloadCache {
    payloads: Mutex<HashMap<String, Arc<FlightRoutePayload>>>,
    handoffs: Mutex<HashMap<String, FlightRoutePayloadHandoff>>,
}

impl FlightRoutePayloadCache {
    pub(super) async fn insert(
        &self,
        cache_key: String,
        payload: FlightRoutePayload,
    ) -> Arc<FlightRoutePayload> {
        let mut payloads = self.payloads.lock().await;
        if let Some(cached) = payloads.get(&cache_key) {
            return Arc::clone(cached);
        }
        if payloads.len() >= MAX_CACHED_ROUTE_PAYLOADS {
            payloads.clear();
        }
        let payload = Arc::new(payload);
        payloads.insert(cache_key, Arc::clone(&payload));
        payload
    }

    pub(super) async fn get(&self, cache_key: &str) -> Option<Arc<FlightRoutePayload>> {
        self.payloads.lock().await.get(cache_key).cloned()
    }

    pub(super) async fn insert_handoff(&self, cache_key: String, payload: Arc<FlightRoutePayload>) {
        let mut handoffs = self.handoffs.lock().await;
        let now = Instant::now();
        Self::retain_live_handoffs(&mut handoffs, now);
        if handoffs.len() >= MAX_ROUTE_PAYLOAD_HANDOFFS {
            handoffs.clear();
        }
        handoffs.insert(
            cache_key,
            FlightRoutePayloadHandoff {
                payload,
                expires_at: now + ROUTE_PAYLOAD_HANDOFF_TTL,
            },
        );
    }

    pub(super) async fn take_handoff(&self, cache_key: &str) -> Option<Arc<FlightRoutePayload>> {
        let mut handoffs = self.handoffs.lock().await;
        let now = Instant::now();
        Self::retain_live_handoffs(&mut handoffs, now);
        handoffs.remove(cache_key).map(|handoff| handoff.payload)
    }

    fn retain_live_handoffs(
        handoffs: &mut HashMap<String, FlightRoutePayloadHandoff>,
        now: Instant,
    ) {
        handoffs.retain(|_, handoff| handoff.expires_at > now);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::RecordBatch;
    use arrow_schema::Schema;

    use super::*;

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

    fn test_payload() -> FlightRoutePayload {
        let schema = Arc::new(Schema::empty());
        let batch = RecordBatch::new_empty(schema);
        FlightRoutePayload::from_batches_with_app_metadata(&[batch], Vec::new())
            .unwrap_or_else(|error| panic!("test payload: {error}"))
    }
}
