use serde::{Deserialize, Serialize};

use super::flowhub_contract::FlowhubStructureContract;

/// Root `[flowhub]` table for one Flowhub library contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowhubRootMetadata {
    /// Stable Flowhub library name.
    pub name: String,
}

/// Top-level Flowhub root contract anchored at `qianji-flowhub/qianji.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowhubRootManifest {
    /// Flowhub root schema version.
    pub version: u64,
    /// Flowhub root metadata.
    pub flowhub: FlowhubRootMetadata,
    /// Registered child graph nodes and their required filesystem surfaces.
    pub contract: FlowhubStructureContract,
}
