//! Capability version contracts used for compatibility checks.

use serde::{Deserialize, Serialize};

/// Stable contract-version marker used by generic plugin-runtime records.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractVersion(pub String);
