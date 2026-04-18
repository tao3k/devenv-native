mod chat;
mod init;

use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::config::RuntimeSettings;
use crate::session::ChatMessage;

use super::backend::LlmBackendMode;
#[cfg(feature = "agent-provider-litellm")]
use super::compat::litellm::LiteLlmRuntime;
use super::providers::{LiteLlmProviderMode, LiteLlmWireApi};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LlmInFlightSnapshot {
    pub max_in_flight: usize,
    pub available_permits: usize,
    pub in_flight: usize,
    pub saturation_pct: u8,
}

/// LLM client for chat completions.
pub struct LlmClient {
    client: reqwest::Client,
    inference_url: String,
    #[cfg(feature = "agent-provider-litellm")]
    inference_api_base: String,
    model: String,
    api_key: Option<String>,
    backend_mode: LlmBackendMode,
    litellm_provider_mode: LiteLlmProviderMode,
    litellm_wire_api: LiteLlmWireApi,
    #[cfg(feature = "agent-provider-litellm")]
    litellm_api_key_env: String,
    #[cfg(feature = "agent-provider-litellm")]
    minimax_api_base: String,
    inference_timeout_secs: u64,
    inference_max_tokens: Option<u32>,
    inference_max_in_flight: Option<usize>,
    in_flight_gate: Option<Arc<Semaphore>>,
    #[cfg(feature = "agent-provider-litellm")]
    litellm_runtime: LiteLlmRuntime,
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

pub(crate) fn enforce_tool_message_integrity_for_tests(
    messages: Vec<ChatMessage>,
) -> (
    Vec<ChatMessage>,
    super::protocol::ToolMessageIntegrityReport,
) {
    super::protocol::enforce_tool_message_integrity(messages)
}

fn compute_saturation_pct(in_flight: usize, max_in_flight: usize) -> u8 {
    if max_in_flight == 0 {
        return 0;
    }
    let ratio = in_flight.saturating_mul(100) / max_in_flight;
    u8::try_from(ratio).unwrap_or(100).min(100)
}

pub(crate) fn test_resolve_backend_mode_for_inference_url(
    runtime_settings: &RuntimeSettings,
    inference_url: &str,
    env_backend_raw: Option<&str>,
) -> (LlmBackendMode, &'static str) {
    init::test_resolve_backend_mode_for_inference_url(
        runtime_settings,
        inference_url,
        env_backend_raw,
    )
}
