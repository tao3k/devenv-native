//! Parser-owned addressed-target contract with optional scoped fragment parts.

use serde::{Deserialize, Serialize};

/// Parser-owned reusable target plus scoped-address contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AddressedTarget {
    /// Optional note or resource target without any address fragment.
    #[serde(default)]
    pub target: Option<String>,
    /// Optional structural address inside the target note or the current note.
    #[serde(default)]
    pub target_address: Option<String>,
}

impl AddressedTarget {
    #[must_use]
    pub(crate) fn new(target: Option<String>, target_address: Option<String>) -> Self {
        Self {
            target,
            target_address,
        }
    }
}
