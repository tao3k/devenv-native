//! Capability selector records for querying available Wendao features.

use serde::{Deserialize, Serialize};

use crate::ids::{CapabilityId, PluginId};

/// Runtime selector that binds one capability to one provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginProviderSelector {
    /// Stable capability identifier.
    pub capability_id: CapabilityId,
    /// Stable plugin identifier.
    pub provider: PluginId,
}
