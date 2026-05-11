//! Artifact selection contracts used by command and search surfaces.

use serde::{Deserialize, Serialize};

use crate::ids::{ArtifactId, PluginId};

/// Runtime selector that binds one artifact kind to one plugin provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginArtifactSelector {
    /// Stable plugin identifier.
    pub plugin_id: PluginId,
    /// Stable artifact identifier.
    pub artifact_id: ArtifactId,
}
