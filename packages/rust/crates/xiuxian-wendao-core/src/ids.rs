//! Shared identifier contracts for Wendao request and response payloads.

use serde::{Deserialize, Serialize};

/// Stable plugin identifier used by generic plugin-runtime contracts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PluginId(pub String);

/// Stable capability identifier used by runtime provider selection.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityId(pub String);

/// Stable artifact identifier used by generic artifact resolution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactId(pub String);
