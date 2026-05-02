//! Embedding client runtime types and tuning constants.

use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::embedding::{EmbeddingBackendMode, EmbeddingCache};

pub(super) const DEFAULT_EMBED_CACHE_TTL_SECS: u64 = 900;
pub(super) const MAX_EMBED_CACHE_TTL_SECS: u64 = 86_400;
pub(super) const DEFAULT_EMBED_CACHE_MAX_ENTRIES: usize = 4_096;
pub(super) const MAX_EMBED_CACHE_MAX_ENTRIES: usize = 65_536;
pub(super) const DEFAULT_EMBED_BATCH_MAX_SIZE: usize = 128;
pub(super) const MAX_EMBED_BATCH_MAX_SIZE: usize = 8_192;
pub(super) const DEFAULT_EMBED_BATCH_MAX_CONCURRENCY: usize = 1;
pub(super) const MAX_EMBED_BATCH_MAX_CONCURRENCY: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Current embedding request concurrency snapshot.
pub struct EmbeddingInFlightSnapshot {
    pub max_in_flight: usize,
    pub available_permits: usize,
    pub in_flight: usize,
    pub saturation_pct: u8,
}

/// Embedding client runtime.
pub struct EmbeddingClient {
    pub(super) client: reqwest::Client,
    pub(super) base_url: String,
    pub(super) cache: EmbeddingCache,
    pub(super) backend_mode: EmbeddingBackendMode,
    pub(super) backend_source: &'static str,
    #[cfg(feature = "agent-provider-litellm")]
    pub(super) timeout_secs: u64,
    pub(super) max_in_flight: Option<usize>,
    pub(super) in_flight_gate: Option<Arc<Semaphore>>,
    pub(super) batch_max_size: usize,
    pub(super) batch_max_concurrency: usize,
    pub(super) default_model: Option<String>,
    #[cfg(feature = "agent-provider-litellm")]
    pub(super) litellm_api_key: Option<String>,
}

#[derive(Clone)]
pub(super) struct EmbeddingDispatchRuntime {
    pub(super) client: reqwest::Client,
    pub(super) base_url: String,
    pub(super) backend_mode: EmbeddingBackendMode,
    pub(super) backend_source: &'static str,
    #[cfg(feature = "agent-provider-litellm")]
    pub(super) timeout_secs: u64,
    pub(super) max_in_flight: Option<usize>,
    pub(super) in_flight_gate: Option<Arc<Semaphore>>,
    #[cfg(feature = "agent-provider-litellm")]
    pub(super) litellm_api_key: Option<String>,
}

impl EmbeddingClient {
    /// Return current in-flight permit usage snapshot when throttling is enabled.
    #[must_use]
    pub fn in_flight_snapshot(&self) -> Option<EmbeddingInFlightSnapshot> {
        let max_in_flight = self.max_in_flight?;
        let available_permits = self.in_flight_gate.as_ref().map_or(max_in_flight, |gate| {
            gate.available_permits().min(max_in_flight)
        });
        let in_flight = max_in_flight.saturating_sub(available_permits);
        let saturation_pct = compute_saturation_pct(in_flight, max_in_flight);
        Some(EmbeddingInFlightSnapshot {
            max_in_flight,
            available_permits,
            in_flight,
            saturation_pct,
        })
    }
}

fn compute_saturation_pct(in_flight: usize, max_in_flight: usize) -> u8 {
    if max_in_flight == 0 {
        return 0;
    }
    let ratio = in_flight.saturating_mul(100) / max_in_flight;
    u8::try_from(ratio).unwrap_or(100).min(100)
}
