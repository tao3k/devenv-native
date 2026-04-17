use std::path::Path;

use serde::{Deserialize, Serialize};
use xiuxian_wendao_core::{
    artifacts::{PluginArtifactPayload, PluginArtifactSelector},
    capabilities::{ContractVersion, PluginCapabilityBinding},
    transport::PluginTransportEndpoint,
};
use xiuxian_wendao_runtime::{
    artifacts::render_plugin_artifact_toml_for_selector_with,
    transport::negotiate_flight_transport_client_from_bindings,
};

use super::launch::LinkGraphJuliaAnalyzerLaunchManifest;
use super::runtime::LinkGraphJuliaRerankRuntimeConfig;
use super::selectors::julia_deployment_artifact_selector;

/// Default artifact-schema version for Julia deployment inspection payloads.
pub const DEFAULT_JULIA_DEPLOYMENT_ARTIFACT_SCHEMA_VERSION: &str = "v1";

/// Serializable deployment artifact for a resolved Julia rerank service.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkGraphJuliaDeploymentArtifact {
    /// Artifact-level schema version for deployment inspection surfaces.
    pub artifact_schema_version: String,
    /// RFC3339 timestamp recording when the deployment artifact was rendered.
    pub generated_at: String,
    /// Resolved Julia service base URL.
    pub base_url: Option<String>,
    /// Arrow Flight route expected by the service.
    pub route: Option<String>,
    /// Health-check route expected by the service.
    pub health_route: Option<String>,
    /// `WendaoArrow` schema version expected by Rust.
    pub schema_version: Option<String>,
    /// Optional request timeout in seconds.
    pub timeout_secs: Option<u64>,
    /// Resolved analyzer launch manifest.
    pub launch: LinkGraphJuliaAnalyzerLaunchManifest,
}

impl LinkGraphJuliaDeploymentArtifact {
    /// Render the deployment artifact as pretty TOML.
    ///
    /// # Errors
    ///
    /// Returns an error when the deployment artifact cannot be serialized into
    /// TOML.
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Render the deployment artifact as pretty JSON.
    ///
    /// # Errors
    ///
    /// Returns an error when the deployment artifact cannot be serialized into
    /// JSON.
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Persist the deployment artifact to a TOML file.
    ///
    /// Parent directories are created when they do not yet exist.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails, when parent directories
    /// cannot be created, or when the artifact file cannot be written.
    pub fn write_toml_file<P>(&self, path: P) -> std::io::Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }

        let encoded = self.to_toml_string().map_err(std::io::Error::other)?;
        std::fs::write(path, encoded)
    }

    /// Persist the deployment artifact to a JSON file.
    ///
    /// Parent directories are created when they do not yet exist.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails, when parent directories
    /// cannot be created, or when the artifact file cannot be written.
    pub fn write_json_file<P>(&self, path: P) -> std::io::Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }

        let encoded = self.to_json_string().map_err(std::io::Error::other)?;
        std::fs::write(path, encoded)
    }
}

impl From<PluginArtifactPayload> for LinkGraphJuliaDeploymentArtifact {
    fn from(value: PluginArtifactPayload) -> Self {
        let artifact_schema_version = value.artifact_schema_version.0.clone();
        let PluginTransportEndpoint {
            base_url,
            route,
            health_route,
            timeout_secs,
            max_in_flight_requests: _,
        } = value.endpoint.unwrap_or_default();

        Self {
            artifact_schema_version,
            generated_at: value.generated_at,
            base_url,
            route,
            health_route,
            schema_version: value.schema_version.or({
                Some(match value.artifact_schema_version {
                    ContractVersion(version) => version,
                })
            }),
            timeout_secs,
            launch: value.launch.unwrap_or_default().into(),
        }
    }
}

impl From<LinkGraphJuliaDeploymentArtifact> for PluginArtifactPayload {
    fn from(value: LinkGraphJuliaDeploymentArtifact) -> Self {
        let selector = julia_deployment_artifact_selector();

        Self {
            plugin_id: selector.plugin_id,
            artifact_id: selector.artifact_id,
            artifact_schema_version: ContractVersion(value.artifact_schema_version),
            generated_at: value.generated_at,
            endpoint: Some(PluginTransportEndpoint {
                base_url: value.base_url,
                route: value.route,
                health_route: value.health_route,
                timeout_secs: value.timeout_secs,
                max_in_flight_requests: None,
            }),
            schema_version: value.schema_version,
            launch: Some(value.launch.into()),
            selected_transport: None,
            fallback_from: None,
            fallback_reason: None,
        }
    }
}

/// Resolve the Julia deployment artifact payload from the runtime config.
#[must_use]
pub fn resolve_julia_deployment_artifact_payload(
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
) -> PluginArtifactPayload {
    let binding = runtime.rerank_provider_binding();
    attach_plugin_artifact_transport_diagnostics(
        runtime.plugin_artifact_payload(),
        binding.as_ref(),
    )
}

/// Resolve one Julia plugin artifact through the Julia rerank runtime config.
#[must_use]
pub fn resolve_julia_plugin_artifact_payload_for_selector(
    selector: &PluginArtifactSelector,
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
) -> Option<PluginArtifactPayload> {
    (selector == &julia_deployment_artifact_selector())
        .then(|| resolve_julia_deployment_artifact_payload(runtime))
}

/// Render a resolved Julia plugin artifact as pretty TOML.
///
/// # Errors
///
/// Returns an error when the resolved artifact cannot be serialized into TOML.
pub fn render_julia_plugin_artifact_toml_for_selector(
    selector: &PluginArtifactSelector,
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
) -> Result<Option<String>, toml::ser::Error> {
    render_plugin_artifact_toml_for_selector_with(selector, |selector| {
        resolve_julia_plugin_artifact_payload_for_selector(selector, runtime)
    })
}

fn attach_plugin_artifact_transport_diagnostics(
    mut artifact: PluginArtifactPayload,
    binding: Option<&PluginCapabilityBinding>,
) -> PluginArtifactPayload {
    let Some(binding) = binding else {
        return artifact;
    };

    match negotiate_flight_transport_client_from_bindings(std::slice::from_ref(binding)) {
        Ok(Some(transport)) => {
            let selection = transport.selection();
            artifact.selected_transport = Some(selection.selected_transport);
            artifact.fallback_from = selection.fallback_from;
            artifact
                .fallback_reason
                .clone_from(&selection.fallback_reason);
        }
        Ok(None) => {
            artifact.fallback_from = Some(binding.transport);
            artifact.fallback_reason = Some(format!(
                "configured transport {:?} is unavailable because the binding has no base_url",
                binding.transport
            ));
        }
        Err(error) => {
            artifact.fallback_from = Some(binding.transport);
            artifact.fallback_reason = Some(error);
        }
    }

    artifact
}
