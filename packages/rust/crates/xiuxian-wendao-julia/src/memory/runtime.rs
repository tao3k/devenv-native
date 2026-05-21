//! Runtime registration for Julia memory compute plugin surfaces.

use xiuxian_wendao_core::{
    capabilities::{ContractVersion, PluginCapabilityBinding, PluginProviderSelector},
    ids::{CapabilityId, PluginId},
    repo_intelligence::RepoIntelligenceError,
    transport::{PluginTransportEndpoint, PluginTransportKind},
};
use xiuxian_wendao_runtime::{
    config::MemoryJuliaComputeRuntimeConfig,
    transport::{
        normalize_flight_route, validate_flight_max_in_flight_requests,
        validate_flight_schema_version, validate_flight_timeout_secs,
    },
};

use super::profile::MemoryJuliaComputeProfile;

/// Build one generic capability binding for a staged memory Julia compute profile.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the runtime config contains invalid
/// provider identity, route, schema version, or timeout values.
pub fn build_memory_julia_compute_binding(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    profile: MemoryJuliaComputeProfile,
) -> Result<Option<PluginCapabilityBinding>, RepoIntelligenceError> {
    if !runtime.enabled {
        return Ok(None);
    }

    let provider = normalized_provider_id(runtime)?;
    let base_url = normalized_base_url(runtime)?;
    let health_route = normalized_health_route(runtime)?;
    let route = normalize_flight_route(route_for_profile(runtime, profile)).map_err(|error| {
        RepoIntelligenceError::ConfigLoad {
            message: format!(
                "memory Julia compute profile `{}` has invalid route: {error}",
                profile.profile_id()
            ),
        }
    })?;
    let schema_version =
        validate_flight_schema_version(&runtime.schema_version).map_err(|error| {
            RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "memory Julia compute profile `{}` has invalid schema version `{}`: {error}",
                    profile.profile_id(),
                    runtime.schema_version
                ),
            }
        })?;
    let timeout_secs =
        validate_flight_timeout_secs(runtime.timeout_secs.value()).map_err(|error| {
            RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "memory Julia compute profile `{}` has invalid timeout `{}`: {error}",
                    profile.profile_id(),
                    runtime.timeout_secs.value()
                ),
            }
        })?;
    let max_in_flight_requests = validate_flight_max_in_flight_requests(
        runtime.max_in_flight_requests,
    )
    .map_err(|error| RepoIntelligenceError::ConfigLoad {
        message: format!(
            "memory Julia compute profile `{}` has invalid max_in_flight_requests `{}`: {error}",
            profile.profile_id(),
            runtime.max_in_flight_requests
        ),
    })?;

    Ok(Some(PluginCapabilityBinding {
        selector: PluginProviderSelector {
            capability_id: CapabilityId(profile.capability_id().to_string()),
            provider: PluginId(provider),
        },
        endpoint: PluginTransportEndpoint {
            base_url: Some(base_url),
            route: Some(route),
            health_route,
            timeout_secs: Some(timeout_secs),
            max_in_flight_requests: Some(u64::try_from(max_in_flight_requests).map_err(|_| {
                RepoIntelligenceError::ConfigLoad {
                    message: format!(
                        "memory Julia compute profile `{}` max_in_flight_requests exceeded u64",
                        profile.profile_id()
                    ),
                }
            })?),
        },
        launch: None,
        transport: PluginTransportKind::ArrowFlight,
        contract_version: ContractVersion(schema_version),
    }))
}

/// Build one binding per staged memory Julia compute profile.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the runtime config contains invalid
/// route, schema version, provider identity, or timeout values.
pub fn build_memory_julia_compute_bindings(
    runtime: &MemoryJuliaComputeRuntimeConfig,
) -> Result<Vec<PluginCapabilityBinding>, RepoIntelligenceError> {
    MemoryJuliaComputeProfile::ALL
        .into_iter()
        .filter_map(|profile| build_memory_julia_compute_binding(runtime, profile).transpose())
        .collect()
}

fn normalized_provider_id(
    runtime: &MemoryJuliaComputeRuntimeConfig,
) -> Result<String, RepoIntelligenceError> {
    let provider = runtime.plugin_id.as_str().trim();
    if provider.is_empty() {
        return Err(RepoIntelligenceError::ConfigLoad {
            message: "memory Julia compute plugin_id must not be blank".to_string(),
        });
    }
    Ok(provider.to_string())
}

fn normalized_base_url(
    runtime: &MemoryJuliaComputeRuntimeConfig,
) -> Result<String, RepoIntelligenceError> {
    let base_url = runtime.base_url.trim();
    if base_url.is_empty() {
        return Err(RepoIntelligenceError::ConfigLoad {
            message: "memory Julia compute base_url must not be blank".to_string(),
        });
    }
    Ok(base_url.to_string())
}

fn route_for_profile(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    profile: MemoryJuliaComputeProfile,
) -> &str {
    match profile {
        MemoryJuliaComputeProfile::EpisodicRecall => runtime.routes.episodic_recall.as_str(),
        MemoryJuliaComputeProfile::MemoryGateScore => runtime.routes.memory_gate_score.as_str(),
        MemoryJuliaComputeProfile::MemoryPlanTuning => runtime.routes.memory_plan_tuning.as_str(),
        MemoryJuliaComputeProfile::MemoryCalibration => runtime.routes.memory_calibration.as_str(),
    }
}

fn normalized_health_route(
    runtime: &MemoryJuliaComputeRuntimeConfig,
) -> Result<Option<String>, RepoIntelligenceError> {
    let Some(health_route) = runtime.health_route.as_deref() else {
        return Ok(None);
    };
    let trimmed = health_route.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    normalize_flight_route(trimmed)
        .map(Some)
        .map_err(|error| RepoIntelligenceError::ConfigLoad {
            message: format!("memory Julia compute health_route is invalid: {error}"),
        })
}

#[cfg(test)]
#[path = "../../tests/unit/memory/runtime.rs"]
mod tests;
