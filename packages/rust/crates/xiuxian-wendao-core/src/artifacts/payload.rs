//! Typed payload envelopes for persisted Wendao artifacts.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::artifacts::PluginLaunchSpec;
use crate::capabilities::ContractVersion;
use crate::ids::{ArtifactId, PluginId};
use crate::transport::{PluginTransportEndpoint, PluginTransportKind};

/// Generic artifact payload returned by plugin-artifact resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginArtifactPayload {
    /// Owner plugin id.
    pub plugin_id: PluginId,
    /// Artifact kind id.
    pub artifact_id: ArtifactId,
    /// Artifact payload schema version.
    pub artifact_schema_version: ContractVersion,
    /// RFC3339 generation timestamp.
    pub generated_at: String,
    /// Optional runtime endpoint carried by the artifact payload.
    pub endpoint: Option<PluginTransportEndpoint>,
    /// Optional provider schema version carried by the artifact payload.
    pub schema_version: Option<String>,
    /// Optional launch metadata carried by the artifact payload.
    pub launch: Option<PluginLaunchSpec>,
    /// Runtime-selected transport surfaced through outward inspection payloads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_transport: Option<PluginTransportKind>,
    /// Higher-preference transport that was skipped before selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_from: Option<PluginTransportKind>,
    /// Reason the runtime fell back from a higher-preference transport.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
}

impl PluginArtifactPayload {
    /// Render the plugin artifact as pretty TOML.
    ///
    /// # Errors
    ///
    /// Returns an error when the artifact cannot be serialized into TOML.
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Render the plugin artifact as pretty JSON.
    ///
    /// # Errors
    ///
    /// Returns an error when the artifact cannot be serialized into JSON.
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Persist the plugin artifact to a TOML file.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails, parent directories cannot be
    /// created, or the file cannot be written.
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

    /// Persist the plugin artifact to a JSON file.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails, parent directories cannot be
    /// created, or the file cannot be written.
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

#[cfg(test)]
#[path = "../../tests/unit/artifacts/payload.rs"]
mod tests;
