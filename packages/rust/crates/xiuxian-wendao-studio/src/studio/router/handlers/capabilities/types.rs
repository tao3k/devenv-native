//! Owns the Studio handlers capabilities types surface.

use serde::Deserialize;
use xiuxian_wendao_core::{
    artifacts::PluginArtifactSelector,
    ids::{ArtifactId, PluginId},
};

use xiuxian_wendao::zhenfa_router::native::WendaoPluginArtifactOutputFormat;

/// Path parameters for Studio generic plugin artifact inspection.
#[derive(Debug, Clone, Deserialize)]
pub struct PluginArtifactPath {
    /// Stable plugin identifier.
    pub plugin_id: String,
    /// Stable artifact identifier.
    pub artifact_id: String,
}

impl From<PluginArtifactPath> for PluginArtifactSelector {
    fn from(value: PluginArtifactPath) -> Self {
        Self {
            plugin_id: PluginId(value.plugin_id),
            artifact_id: ArtifactId(value.artifact_id),
        }
    }
}

/// Query parameters for Studio generic plugin artifact inspection.
#[derive(Debug, Default, Deserialize)]
pub struct PluginArtifactQuery {
    /// Optional response format. Defaults to structured JSON.
    #[serde(default)]
    pub format: Option<WendaoPluginArtifactOutputFormat>,
}
