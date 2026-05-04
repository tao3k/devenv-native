//! Studio-owned plugin artifact inspection contracts.

use serde::{Deserialize, Serialize};
use specta::{Type, TypeCollection};
#[cfg(feature = "local-runtime")]
use xiuxian_wendao_core::{
    artifacts::{PluginArtifactPayload, PluginLaunchSpec},
    transport::PluginTransportKind,
};

/// Studio-visible generic plugin launch manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct UiPluginLaunchSpec {
    /// Launcher path relative to the repository root.
    pub launcher_path: String,
    /// Ordered provider-owned CLI args.
    pub args: Vec<String>,
}

#[cfg(feature = "local-runtime")]
impl From<PluginLaunchSpec> for UiPluginLaunchSpec {
    fn from(value: PluginLaunchSpec) -> Self {
        Self {
            launcher_path: value.launcher_path,
            args: value.args,
        }
    }
}

/// Studio-visible generic plugin transport kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum UiPluginTransportKind {
    /// Generic plugin transport over Arrow Flight.
    ArrowFlight,
}

#[cfg(feature = "local-runtime")]
impl From<PluginTransportKind> for UiPluginTransportKind {
    fn from(value: PluginTransportKind) -> Self {
        match value {
            PluginTransportKind::ArrowFlight => Self::ArrowFlight,
        }
    }
}

/// Studio-visible generic plugin artifact inspection payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct UiPluginArtifact {
    /// Owner plugin id.
    pub plugin_id: String,
    /// Artifact kind id.
    pub artifact_id: String,
    /// Artifact-level schema version for inspection surfaces.
    pub artifact_schema_version: String,
    /// RFC3339 timestamp recording when the artifact was rendered.
    pub generated_at: String,
    /// Resolved provider service base URL.
    pub base_url: Option<String>,
    /// Request route expected by the provider.
    pub route: Option<String>,
    /// Health-check route expected by the provider.
    pub health_route: Option<String>,
    /// Optional request timeout in seconds.
    pub timeout_secs: Option<u64>,
    /// Optional provider schema version.
    pub schema_version: Option<String>,
    /// Optional launch manifest for managed providers.
    pub launch: Option<UiPluginLaunchSpec>,
    /// Runtime-selected transport surfaced by the current negotiation boundary.
    pub selected_transport: Option<UiPluginTransportKind>,
    /// Higher-preference transport skipped before selection.
    pub fallback_from: Option<UiPluginTransportKind>,
    /// Reason the runtime fell back from a higher-preference transport.
    pub fallback_reason: Option<String>,
}

#[cfg(feature = "local-runtime")]
impl From<PluginArtifactPayload> for UiPluginArtifact {
    fn from(value: PluginArtifactPayload) -> Self {
        let endpoint = value.endpoint;
        Self {
            plugin_id: value.plugin_id.0,
            artifact_id: value.artifact_id.0,
            artifact_schema_version: value.artifact_schema_version.0,
            generated_at: value.generated_at,
            base_url: endpoint
                .as_ref()
                .and_then(|endpoint| endpoint.base_url.clone()),
            route: endpoint
                .as_ref()
                .and_then(|endpoint| endpoint.route.clone()),
            health_route: endpoint
                .as_ref()
                .and_then(|endpoint| endpoint.health_route.clone()),
            timeout_secs: endpoint.as_ref().and_then(|endpoint| endpoint.timeout_secs),
            schema_version: value.schema_version,
            launch: value.launch.map(Into::into),
            selected_transport: value.selected_transport.map(Into::into),
            fallback_from: value.fallback_from.map(Into::into),
            fallback_reason: value.fallback_reason,
        }
    }
}

/// Build the plugin-only Studio Specta type collection.
#[must_use]
pub fn studio_type_collection() -> TypeCollection {
    TypeCollection::default()
        .register::<UiPluginArtifact>()
        .register::<UiPluginLaunchSpec>()
}
