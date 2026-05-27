use crate::valkey::queue::{ValkeyLeaseId, ValkeyWorkerId};

/// A typed lease ownership fact returned by Valkey scripts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValkeyLeaseOwnership {
    lease_id: ValkeyLeaseId,
    worker_id: ValkeyWorkerId,
}

impl ValkeyLeaseOwnership {
    /// Creates a typed ownership fact.
    #[must_use]
    pub const fn new(lease_id: ValkeyLeaseId, worker_id: ValkeyWorkerId) -> Self {
        Self {
            lease_id,
            worker_id,
        }
    }

    /// Borrows the lease id.
    #[must_use]
    pub const fn lease_id(&self) -> &ValkeyLeaseId {
        &self.lease_id
    }

    /// Borrows the worker id.
    #[must_use]
    pub const fn worker_id(&self) -> &ValkeyWorkerId {
        &self.worker_id
    }
}

impl std::fmt::Display for ValkeyLeaseOwnership {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "lease `{}` worker `{}`",
            self.lease_id.as_str(),
            self.worker_id.as_str()
        )
    }
}

/// Valkey command errors.
#[derive(Debug, thiserror::Error)]
pub enum ValkeyStoreError {
    /// A required identifier was blank.
    #[error("{field} must not be blank")]
    BlankId {
        /// Field that was blank.
        field: &'static str,
    },
    /// A positive TTL was required.
    #[error("{field} must be greater than zero")]
    NonPositiveTtl {
        /// Field that failed validation.
        field: &'static str,
    },
    /// Redis/Valkey command execution failed.
    #[error("{operation} failed: {message}")]
    Storage {
        /// Operation label.
        operation: &'static str,
        /// Backend message.
        message: String,
    },
    /// A lease was owned by a different worker.
    #[error("{ownership} is not active for this queue operation")]
    LeaseNotOwned {
        /// Typed lease ownership fact returned by the script.
        ownership: ValkeyLeaseOwnership,
    },
    /// Backend returned an unexpected script result.
    #[error("unexpected Valkey lease script result {result}")]
    UnexpectedLeaseScriptResult {
        /// Script result.
        result: i64,
    },
}

pub(crate) fn non_blank(value: String, field: &'static str) -> Result<String, ValkeyStoreError> {
    if value.trim().is_empty() {
        return Err(ValkeyStoreError::BlankId { field });
    }
    Ok(value)
}

pub(crate) fn validate_positive_ttl(
    field: &'static str,
    ttl_ms: u64,
) -> Result<(), ValkeyStoreError> {
    if ttl_ms == 0 {
        return Err(ValkeyStoreError::NonPositiveTtl { field });
    }
    Ok(())
}
