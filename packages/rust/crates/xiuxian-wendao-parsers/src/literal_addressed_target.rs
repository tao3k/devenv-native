//! Source-preserved addressed-target contract shared across parser formats.

use crate::AddressedTarget;
use serde::{Deserialize, Serialize};

/// Parser-owned source-preserved addressed target shared across formats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiteralAddressedTarget {
    /// Shared parser-owned target plus optional structural address.
    #[serde(default)]
    #[serde(flatten)]
    pub addressed_target: AddressedTarget,
    /// Original literal slice from the source document.
    pub original: String,
}

impl LiteralAddressedTarget {
    #[must_use]
    pub(crate) fn new(addressed_target: AddressedTarget, original: String) -> Self {
        Self {
            addressed_target,
            original,
        }
    }
}
