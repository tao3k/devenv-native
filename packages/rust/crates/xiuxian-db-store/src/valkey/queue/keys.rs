//! Structured Valkey queue key derivation.

use crate::valkey::{
    error::{ValkeyStoreError, non_blank},
    queue::ValkeyQueueEntryId,
};

/// Structured queue keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValkeyQueueKeys {
    pub(super) pending_key: String,
    pub(super) lease_deadlines_key: String,
    pub(super) payload_prefix: String,
    pub(super) lease_prefix: String,
}

impl ValkeyQueueKeys {
    /// Creates a structured queue key set.
    ///
    /// # Errors
    ///
    /// Returns an error when any key or prefix is blank.
    pub fn new(
        pending_key: impl Into<String>,
        lease_deadlines_key: impl Into<String>,
        payload_prefix: impl Into<String>,
        lease_prefix: impl Into<String>,
    ) -> Result<Self, ValkeyStoreError> {
        Ok(Self {
            pending_key: non_blank(pending_key.into(), "pending_key")?,
            lease_deadlines_key: non_blank(lease_deadlines_key.into(), "lease_deadlines_key")?,
            payload_prefix: non_blank(payload_prefix.into(), "payload_prefix")?,
            lease_prefix: non_blank(lease_prefix.into(), "lease_prefix")?,
        })
    }

    /// Pending sorted-set key.
    #[must_use]
    pub fn pending_key(&self) -> &str {
        self.pending_key.as_str()
    }

    /// Lease deadline sorted-set key.
    #[must_use]
    pub fn lease_deadlines_key(&self) -> &str {
        self.lease_deadlines_key.as_str()
    }

    /// Payload hash key for an entry.
    #[must_use]
    pub fn payload_key(&self, entry_id: &ValkeyQueueEntryId) -> String {
        format!("{}{}", self.payload_prefix, entry_id.as_str())
    }

    /// Lease hash key for an entry.
    #[must_use]
    pub fn lease_key(&self, entry_id: &ValkeyQueueEntryId) -> String {
        format!("{}{}", self.lease_prefix, entry_id.as_str())
    }
}
