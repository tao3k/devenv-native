//! LLM client runtime types.

use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::llm::LlmBackendMode;
#[cfg(feature = "agent-provider-litellm")]
use crate::llm::compat::litellm::LiteLlmRuntime;
#[cfg(feature = "agent-provider-litellm")]
use crate::llm::providers::{LiteLlmProviderMode, LiteLlmWireApi};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Current LLM request concurrency snapshot.
pub struct LlmInFlightSnapshot {
    pub max_in_flight: usize,
    pub available_permits: usize,
    pub in_flight: usize,
    pub saturation_pct: u8,
}

/// LLM client for chat completions.
pub struct LlmClient {
    pub(super) client: reqwest::Client,
    pub(super) inference_url: String,
    #[cfg(feature = "agent-provider-litellm")]
    pub(super) inference_api_base: String,
    pub(super) model: String,
    pub(super) api_key: Option<String>,
    pub(super) backend_mode: LlmBackendMode,
    #[cfg(feature = "agent-provider-litellm")]
    pub(super) litellm_provider_mode: LiteLlmProviderMode,
    #[cfg(feature = "agent-provider-litellm")]
    pub(super) litellm_wire_api: LiteLlmWireApi,
    #[cfg(feature = "agent-provider-litellm")]
    pub(super) litellm_api_key_env: String,
    #[cfg(feature = "agent-provider-litellm")]
    pub(super) minimax_api_base: String,
    #[cfg(feature = "agent-provider-litellm")]
    pub(super) inference_timeout_secs: u64,
    pub(super) inference_max_tokens: Option<u32>,
    pub(super) inference_max_in_flight: Option<usize>,
    pub(super) in_flight_gate: Option<Arc<Semaphore>>,
    #[cfg(feature = "agent-provider-litellm")]
    pub(super) litellm_runtime: LiteLlmRuntime,
}

impl LlmClient {
    #[must_use]
    pub fn in_flight_snapshot(&self) -> Option<LlmInFlightSnapshot> {
        let max_in_flight = self.inference_max_in_flight?;
        let available_permits = self.in_flight_gate.as_ref().map_or(max_in_flight, |gate| {
            gate.available_permits().min(max_in_flight)
        });
        let in_flight = max_in_flight.saturating_sub(available_permits);
        let saturation_pct = compute_saturation_pct(in_flight, max_in_flight);
        Some(LlmInFlightSnapshot {
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
