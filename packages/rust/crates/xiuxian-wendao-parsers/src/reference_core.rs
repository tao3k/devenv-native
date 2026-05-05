//! Shared normalized reference core used by Markdown reference variants.

use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::{AddressedTarget, LiteralAddressedTarget};

/// Parser-owned source-preserved reference payload shared across formats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceCore<Kind> {
    /// Format-local reference syntax kind carried by the shared payload.
    pub kind: Kind,
    /// Shared parser-owned addressed target plus original literal.
    #[serde(flatten)]
    pub literal_addressed_target: LiteralAddressedTarget,
}

impl<Kind> ReferenceCore<Kind> {
    #[must_use]
    pub(crate) fn new(kind: Kind, addressed_target: AddressedTarget, original: String) -> Self {
        Self {
            kind,
            literal_addressed_target: LiteralAddressedTarget::new(addressed_target, original),
        }
    }
}

impl<Kind> AsRef<LiteralAddressedTarget> for ReferenceCore<Kind> {
    fn as_ref(&self) -> &LiteralAddressedTarget {
        &self.literal_addressed_target
    }
}

impl<Kind> Deref for ReferenceCore<Kind> {
    type Target = LiteralAddressedTarget;

    fn deref(&self) -> &Self::Target {
        &self.literal_addressed_target
    }
}
