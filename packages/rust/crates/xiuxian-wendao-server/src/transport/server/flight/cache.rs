use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use super::payload::FlightRoutePayload;

const MAX_CACHED_ROUTE_PAYLOADS: usize = 128;

#[derive(Debug, Default)]
pub(super) struct FlightRoutePayloadCache {
    payloads: Mutex<HashMap<String, Arc<FlightRoutePayload>>>,
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
}
