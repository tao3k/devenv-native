use chrono::Utc;
use serde::{Deserialize, Serialize};
use xiuxian_wendao_core::{
    artifacts::{PluginArtifactPayload, PluginLaunchSpec},
    capabilities::{ContractVersion, PluginCapabilityBinding},
    transport::{PluginTransportEndpoint, PluginTransportKind},
};

use super::artifact::{
    DEFAULT_JULIA_DEPLOYMENT_ARTIFACT_SCHEMA_VERSION, LinkGraphJuliaDeploymentArtifact,
};
use super::launch::{
    LinkGraphJuliaAnalyzerLaunchManifest, LinkGraphJuliaAnalyzerServiceDescriptor,
};
use super::paths::{DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH, DEFAULT_JULIA_RERANK_FLIGHT_ROUTE};
use super::selectors::{julia_deployment_artifact_selector, julia_rerank_provider_selector};

/// Runtime knobs for remote Julia rerank over Arrow Flight.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LinkGraphJuliaRerankRuntimeConfig {
    /// Base URL for the WendaoArrow-compatible Julia service.
    pub base_url: Option<String>,
    /// Arrow Flight request route.
    pub route: Option<String>,
    /// Health-check route.
    pub health_route: Option<String>,
    /// `WendaoArrow` schema version expected by the runtime.
    pub schema_version: Option<String>,
    /// Optional request timeout in seconds.
    pub timeout_secs: Option<u64>,
    /// Optional analyzer-owned service mode for generic analyzer launchers.
    pub service_mode: Option<String>,
    /// Optional analyzer-owned TOML path passed to the Julia service launcher.
    pub analyzer_config_path: Option<String>,
    /// Optional analyzer-owned strategy name for local or managed Julia services.
    pub analyzer_strategy: Option<String>,
    /// Optional analyzer vector weight for linear-blend strategies.
    pub vector_weight: Option<f64>,
    /// Optional analyzer similarity weight for linear-blend strategies.
    pub similarity_weight: Option<f64>,
}

impl LinkGraphJuliaRerankRuntimeConfig {
    fn resolved_route(&self) -> String {
        self.route
            .clone()
            .unwrap_or_else(|| DEFAULT_JULIA_RERANK_FLIGHT_ROUTE.to_string())
    }

    /// Return whether the legacy Julia rerank runtime carries any configured provider inputs.
    #[must_use]
    pub fn is_configured(&self) -> bool {
        self.base_url.is_some()
            || self.route.is_some()
            || self.health_route.is_some()
            || self.schema_version.is_some()
            || self.timeout_secs.is_some()
            || self.service_mode.is_some()
            || self.analyzer_config_path.is_some()
            || self.analyzer_strategy.is_some()
            || self.vector_weight.is_some()
            || self.similarity_weight.is_some()
    }

    /// Build provider-owned launch inputs from runtime configuration.
    #[must_use]
    pub fn provider_launch_descriptor(&self) -> LinkGraphJuliaAnalyzerServiceDescriptor {
        LinkGraphJuliaAnalyzerServiceDescriptor {
            service_mode: self.service_mode.clone(),
            analyzer_config_path: self.analyzer_config_path.clone(),
            analyzer_strategy: self.analyzer_strategy.clone(),
            vector_weight: self.vector_weight,
            similarity_weight: self.similarity_weight,
        }
    }

    /// Build the plugin launch specification from runtime configuration.
    #[must_use]
    pub fn plugin_launch_spec(&self) -> PluginLaunchSpec {
        self.provider_launch_descriptor()
            .plugin_launch_spec(DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH)
    }

    /// Build the serializable generic plugin artifact payload from runtime configuration.
    #[must_use]
    pub fn plugin_artifact_payload(&self) -> PluginArtifactPayload {
        let selector = julia_deployment_artifact_selector();

        PluginArtifactPayload {
            plugin_id: selector.plugin_id,
            artifact_id: selector.artifact_id,
            artifact_schema_version: ContractVersion(
                DEFAULT_JULIA_DEPLOYMENT_ARTIFACT_SCHEMA_VERSION.to_string(),
            ),
            generated_at: Utc::now().to_rfc3339(),
            endpoint: Some(PluginTransportEndpoint {
                base_url: self.base_url.clone(),
                route: Some(self.resolved_route()),
                health_route: self.health_route.clone(),
                timeout_secs: self.timeout_secs,
                max_in_flight_requests: None,
            }),
            schema_version: self.schema_version.clone(),
            launch: Some(self.plugin_launch_spec()),
            selected_transport: None,
            fallback_from: None,
            fallback_reason: None,
        }
    }

    /// Normalize the provider runtime config into a generic capability binding.
    #[must_use]
    pub fn rerank_provider_binding(&self) -> Option<PluginCapabilityBinding> {
        self.is_configured()
            .then(|| build_rerank_provider_binding(self))
    }

    /// Build the analyzer-owned launch descriptor from runtime configuration.
    #[must_use]
    pub fn analyzer_service_descriptor(&self) -> LinkGraphJuliaAnalyzerServiceDescriptor {
        self.provider_launch_descriptor()
    }

    /// Build the analyzer launch manifest from runtime configuration.
    #[must_use]
    pub fn analyzer_launch_manifest(&self) -> LinkGraphJuliaAnalyzerLaunchManifest {
        self.plugin_launch_spec().into()
    }

    /// Build the serializable deployment artifact from runtime configuration.
    #[must_use]
    pub fn deployment_artifact(&self) -> LinkGraphJuliaDeploymentArtifact {
        self.plugin_artifact_payload().into()
    }

    /// Normalize the legacy Julia rerank runtime config into a generic capability binding.
    #[must_use]
    pub fn plugin_capability_binding(&self) -> Option<PluginCapabilityBinding> {
        self.rerank_provider_binding()
    }
}

/// Build a generic capability binding from the legacy Julia rerank runtime config.
#[must_use]
pub fn build_rerank_provider_binding(
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
) -> PluginCapabilityBinding {
    PluginCapabilityBinding {
        selector: julia_rerank_provider_selector(),
        endpoint: PluginTransportEndpoint {
            base_url: runtime.base_url.clone(),
            route: Some(runtime.resolved_route()),
            health_route: runtime.health_route.clone(),
            timeout_secs: runtime.timeout_secs,
            max_in_flight_requests: None,
        },
        launch: Some(runtime.plugin_launch_spec()),
        transport: PluginTransportKind::ArrowFlight,
        contract_version: ContractVersion(
            runtime
                .schema_version
                .clone()
                .unwrap_or_else(|| "v1".to_string()),
        ),
    }
}
