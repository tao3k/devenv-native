//! Flight client helpers for Julia memory compute routes.

use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;
use xiuxian_wendao_runtime::{
    config::MemoryJuliaComputeRuntimeConfig,
    transport::{NegotiatedFlightTransportClient, negotiate_flight_transport_client_from_bindings},
};

use crate::wendao::memory::{MemoryJuliaComputeProfile, build_memory_julia_compute_binding};

/// Build one negotiated Arrow Flight client for a memory-family Julia compute
/// profile from runtime config.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the runtime config cannot be
/// converted into a valid transport binding or negotiated Flight client.
pub fn build_memory_julia_compute_flight_transport_client(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    profile: MemoryJuliaComputeProfile,
) -> Result<Option<NegotiatedFlightTransportClient>, RepoIntelligenceError> {
    let Some(binding) = build_memory_julia_compute_binding(runtime, profile)? else {
        return Ok(None);
    };

    negotiate_flight_transport_client_from_bindings(&[binding]).map_err(|error| {
        RepoIntelligenceError::ConfigLoad {
            message: format!(
                "failed to build memory Julia compute Flight client for profile `{}`: {error}",
                profile.profile_id()
            ),
        }
    })
}
