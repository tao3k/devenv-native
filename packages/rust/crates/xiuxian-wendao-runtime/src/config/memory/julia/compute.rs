//! Julia memory compute configuration records for runtime config parsing.

use crate::settings::{first_non_empty, get_setting_bool, get_setting_string, parse_positive_u64};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

/// Default Flight base URL for memory-family Julia compute services.
pub const DEFAULT_MEMORY_JULIA_COMPUTE_BASE_URL: &str = "http://127.0.0.1:8815";
/// Default physical schema version for the memory-family Flight contract.
pub const DEFAULT_MEMORY_JULIA_COMPUTE_SCHEMA_VERSION: &str = "v1";
/// Default plugin identifier for the memory-family Julia compute provider.
pub const DEFAULT_MEMORY_JULIA_COMPUTE_PLUGIN_ID: &str = "wendao.memory";
/// Default timeout for memory-family Julia compute roundtrips.
pub const DEFAULT_MEMORY_JULIA_COMPUTE_TIMEOUT_SECS: u64 = 10;
/// Default in-flight request budget for memory-family Julia compute roundtrips.
pub const DEFAULT_MEMORY_JULIA_COMPUTE_MAX_IN_FLIGHT_REQUESTS: u64 = 32;
/// Default route for episodic recall compute requests.
pub const DEFAULT_MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_ROUTE: &str = "/memory/episodic_recall";
/// Default route for memory gate scoring requests.
pub const DEFAULT_MEMORY_JULIA_COMPUTE_GATE_SCORE_ROUTE: &str = "/memory/gate_score";
/// Default route for memory plan tuning requests.
pub const DEFAULT_MEMORY_JULIA_COMPUTE_PLAN_TUNING_ROUTE: &str = "/memory/plan_tuning";
/// Default route for memory calibration requests.
pub const DEFAULT_MEMORY_JULIA_COMPUTE_CALIBRATION_ROUTE: &str = "/memory/calibration";

/// Fallback behavior for memory-family Julia compute integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MemoryJuliaComputeFallbackMode {
    /// Fall back to the Rust-hosted implementation when Julia compute is unavailable.
    #[default]
    Rust,
}

impl MemoryJuliaComputeFallbackMode {
    fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "rust" => Some(Self::Rust),
            _ => None,
        }
    }
}

/// Transport interaction mode requested from the Julia compute provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MemoryJuliaComputeServiceMode {
    /// Expect stream-capable Flight handlers by default.
    #[default]
    Stream,
    /// Expect table-oriented Flight handlers.
    Table,
}

impl MemoryJuliaComputeServiceMode {
    fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "stream" => Some(Self::Stream),
            "table" => Some(Self::Table),
            _ => None,
        }
    }
}

/// Provider identity for the memory-family Julia compute lane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MemoryJuliaComputePluginId(String);

impl MemoryJuliaComputePluginId {
    /// Creates a plugin identity from a caller-validated value.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the plugin identity as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Consumes this value into its owned string representation.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl Default for MemoryJuliaComputePluginId {
    fn default() -> Self {
        Self(DEFAULT_MEMORY_JULIA_COMPUTE_PLUGIN_ID.to_string())
    }
}

impl From<&str> for MemoryJuliaComputePluginId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for MemoryJuliaComputePluginId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl PartialEq<&str> for MemoryJuliaComputePluginId {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Timeout budget, in seconds, for a memory-family Julia compute roundtrip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MemoryJuliaComputeTimeoutSecs(u64);

impl MemoryJuliaComputeTimeoutSecs {
    /// Creates a timeout value from a caller-validated positive second count.
    #[must_use]
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the timeout as seconds.
    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}

impl Default for MemoryJuliaComputeTimeoutSecs {
    fn default() -> Self {
        Self(DEFAULT_MEMORY_JULIA_COMPUTE_TIMEOUT_SECS)
    }
}

impl From<u64> for MemoryJuliaComputeTimeoutSecs {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl PartialEq<u64> for MemoryJuliaComputeTimeoutSecs {
    fn eq(&self, other: &u64) -> bool {
        self.value() == *other
    }
}

/// Route map for the first `memory` capability family profiles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryJuliaComputeRoutesRuntimeConfig {
    /// Route for read-only episodic recall compute.
    pub episodic_recall: String,
    /// Route for recommendation-only memory gate scoring.
    pub memory_gate_score: String,
    /// Route for advice-only memory plan tuning.
    pub memory_plan_tuning: String,
    /// Route for artifact-only memory calibration.
    pub memory_calibration: String,
}

impl Default for MemoryJuliaComputeRoutesRuntimeConfig {
    fn default() -> Self {
        Self {
            episodic_recall: DEFAULT_MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_ROUTE.to_string(),
            memory_gate_score: DEFAULT_MEMORY_JULIA_COMPUTE_GATE_SCORE_ROUTE.to_string(),
            memory_plan_tuning: DEFAULT_MEMORY_JULIA_COMPUTE_PLAN_TUNING_ROUTE.to_string(),
            memory_calibration: DEFAULT_MEMORY_JULIA_COMPUTE_CALIBRATION_ROUTE.to_string(),
        }
    }
}

/// Runtime-owned memory-family Julia compute configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryJuliaComputeRuntimeConfig {
    /// Enable the external Julia compute lane.
    pub enabled: bool,
    /// Base URL for the memory-family Flight service.
    pub base_url: String,
    /// Physical schema version for request/response transport.
    pub schema_version: String,
    /// Provider identity advertised by the Julia capability manifest.
    pub plugin_id: MemoryJuliaComputePluginId,
    /// Optional family-level health route for the Julia compute service.
    pub health_route: Option<String>,
    /// Transport interaction mode used by the host integration.
    pub service_mode: MemoryJuliaComputeServiceMode,
    /// Optional scenario pack forwarded into the Julia compute lane.
    pub scenario_pack: Option<String>,
    /// Timeout budget for one compute roundtrip.
    pub timeout_secs: MemoryJuliaComputeTimeoutSecs,
    /// Maximum concurrent in-flight compute roundtrips per host transport client.
    pub max_in_flight_requests: u64,
    /// Fallback behavior when Julia compute is unavailable.
    pub fallback_mode: MemoryJuliaComputeFallbackMode,
    /// Whether the host should record shadow drift against the Rust baseline.
    pub shadow_compare: bool,
    /// Profile routes under the memory-family provider.
    pub routes: MemoryJuliaComputeRoutesRuntimeConfig,
}

impl Default for MemoryJuliaComputeRuntimeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: DEFAULT_MEMORY_JULIA_COMPUTE_BASE_URL.to_string(),
            schema_version: DEFAULT_MEMORY_JULIA_COMPUTE_SCHEMA_VERSION.to_string(),
            plugin_id: MemoryJuliaComputePluginId::default(),
            health_route: None,
            service_mode: MemoryJuliaComputeServiceMode::default(),
            scenario_pack: None,
            timeout_secs: MemoryJuliaComputeTimeoutSecs::default(),
            max_in_flight_requests: DEFAULT_MEMORY_JULIA_COMPUTE_MAX_IN_FLIGHT_REQUESTS,
            fallback_mode: MemoryJuliaComputeFallbackMode::default(),
            shadow_compare: true,
            routes: MemoryJuliaComputeRoutesRuntimeConfig::default(),
        }
    }
}

fn resolve_non_empty_string(settings: &Value, dotted_key: &str) -> Option<String> {
    first_non_empty(&[get_setting_string(settings, dotted_key)])
}

/// Resolve `memory.julia_compute` from merged Wendao settings.
#[must_use]
pub fn resolve_memory_julia_compute_runtime_with_settings(
    settings: &Value,
) -> MemoryJuliaComputeRuntimeConfig {
    let mut resolved = MemoryJuliaComputeRuntimeConfig::default();
    apply_memory_julia_compute_endpoint_settings(settings, &mut resolved);
    apply_memory_julia_compute_behavior_settings(settings, &mut resolved);
    apply_memory_julia_compute_route_settings(settings, &mut resolved);
    resolved
}

fn apply_memory_julia_compute_endpoint_settings(
    settings: &Value,
    resolved: &mut MemoryJuliaComputeRuntimeConfig,
) {
    if let Some(enabled) = get_setting_bool(settings, "memory.julia_compute.enabled") {
        resolved.enabled = enabled;
    }

    if let Some(base_url) = resolve_non_empty_string(settings, "memory.julia_compute.base_url") {
        resolved.base_url = base_url;
    }

    if let Some(schema_version) =
        resolve_non_empty_string(settings, "memory.julia_compute.schema_version")
    {
        resolved.schema_version = schema_version;
    }

    if let Some(plugin_id) = resolve_non_empty_string(settings, "memory.julia_compute.plugin_id") {
        resolved.plugin_id = MemoryJuliaComputePluginId::new(plugin_id);
    }

    resolved.health_route = resolve_non_empty_string(settings, "memory.julia_compute.health_route");
}

fn apply_memory_julia_compute_behavior_settings(
    settings: &Value,
    resolved: &mut MemoryJuliaComputeRuntimeConfig,
) {
    if let Some(service_mode) =
        resolve_non_empty_string(settings, "memory.julia_compute.service_mode")
            .as_deref()
            .and_then(MemoryJuliaComputeServiceMode::parse)
    {
        resolved.service_mode = service_mode;
    }

    resolved.scenario_pack =
        resolve_non_empty_string(settings, "memory.julia_compute.scenario_pack");

    if let Some(timeout_secs) =
        resolve_non_empty_string(settings, "memory.julia_compute.timeout_secs")
            .as_deref()
            .and_then(parse_positive_u64)
    {
        resolved.timeout_secs = MemoryJuliaComputeTimeoutSecs::new(timeout_secs);
    }

    if let Some(max_in_flight_requests) =
        resolve_non_empty_string(settings, "memory.julia_compute.max_in_flight_requests")
            .as_deref()
            .and_then(parse_positive_u64)
    {
        resolved.max_in_flight_requests = max_in_flight_requests;
    }

    if let Some(fallback_mode) =
        resolve_non_empty_string(settings, "memory.julia_compute.fallback_mode")
            .as_deref()
            .and_then(MemoryJuliaComputeFallbackMode::parse)
    {
        resolved.fallback_mode = fallback_mode;
    }

    if let Some(shadow_compare) = get_setting_bool(settings, "memory.julia_compute.shadow_compare")
    {
        resolved.shadow_compare = shadow_compare;
    }
}

fn apply_memory_julia_compute_route_settings(
    settings: &Value,
    resolved: &mut MemoryJuliaComputeRuntimeConfig,
) {
    if let Some(route) =
        resolve_non_empty_string(settings, "memory.julia_compute.routes.episodic_recall")
    {
        resolved.routes.episodic_recall = route;
    }

    if let Some(route) =
        resolve_non_empty_string(settings, "memory.julia_compute.routes.memory_gate_score")
    {
        resolved.routes.memory_gate_score = route;
    }

    if let Some(route) =
        resolve_non_empty_string(settings, "memory.julia_compute.routes.memory_plan_tuning")
    {
        resolved.routes.memory_plan_tuning = route;
    }

    if let Some(route) =
        resolve_non_empty_string(settings, "memory.julia_compute.routes.memory_calibration")
    {
        resolved.routes.memory_calibration = route;
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/config/memory/julia/compute.rs"]
mod tests;
