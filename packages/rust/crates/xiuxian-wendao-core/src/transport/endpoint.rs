//! Endpoint configuration for Wendao plugin transport providers.

use serde::{Deserialize, Serialize};

/// Generic runtime endpoint for a plugin capability or artifact provider.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginTransportEndpoint {
    /// Base URL for the provider service.
    pub base_url: Option<String>,
    /// Main request route for the provider capability or artifact.
    pub route: Option<String>,
    /// Health-check route for readiness probes.
    pub health_route: Option<String>,
    /// Optional request timeout in seconds.
    pub timeout_secs: Option<u64>,
    /// Optional maximum concurrent in-flight requests for one transport client.
    pub max_in_flight_requests: Option<u64>,
}
