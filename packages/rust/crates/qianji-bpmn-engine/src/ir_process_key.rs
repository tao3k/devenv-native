//! Public ir process key contracts for BPMN/DMN engine integration.

use std::sync::Arc;

/// Stable process identity metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProcessKey {
    /// Package identifier.
    pub package_id: Arc<str>,
    /// Process identifier.
    pub process_id: Arc<str>,
    /// Spec digest or content hash placeholder.
    pub spec_digest_hex: Arc<str>,
}

impl ProcessKey {
    /// Creates a process identity.
    #[must_use]
    pub fn new(
        package_id: impl AsRef<str>,
        process_id: impl AsRef<str>,
        spec_digest_hex: impl AsRef<str>,
    ) -> Self {
        Self {
            package_id: (Arc::<str>::from(package_id.as_ref())),
            process_id: (Arc::<str>::from(process_id.as_ref())),
            spec_digest_hex: Arc::<str>::from(spec_digest_hex.as_ref()),
        }
    }
}
