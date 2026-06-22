//! Typed Valkey queue identifiers, entries, and lease models.

use crate::valkey::{
    ValkeyLeaseOwnership,
    error::{ValkeyStoreError, non_blank},
};

/// Stable structured queue entry id.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValkeyQueueEntryId(String);

impl ValkeyQueueEntryId {
    /// Creates a non-empty queue entry id.
    ///
    /// # Errors
    ///
    /// Returns an error when the id is blank.
    pub fn new(value: impl Into<String>) -> Result<Self, ValkeyStoreError> {
        Ok(Self(non_blank(value.into(), "entry_id")?))
    }

    /// Borrows the entry id.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Stable Valkey lease id.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValkeyLeaseId(String);

impl ValkeyLeaseId {
    /// Creates a non-empty lease id.
    ///
    /// # Errors
    ///
    /// Returns an error when the id is blank.
    pub fn new(value: impl Into<String>) -> Result<Self, ValkeyStoreError> {
        Ok(Self(non_blank(value.into(), "lease_id")?))
    }

    /// Borrows the lease id.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Stable worker id for Valkey queue leases.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValkeyWorkerId(String);

impl ValkeyWorkerId {
    /// Creates a non-empty worker id.
    ///
    /// # Errors
    ///
    /// Returns an error when the id is blank.
    pub fn new(value: impl Into<String>) -> Result<Self, ValkeyStoreError> {
        Ok(Self(non_blank(value.into(), "worker_id")?))
    }

    /// Borrows the worker id.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// One structured queue entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValkeyStructuredQueueEntry<'a> {
    entry_id: ValkeyQueueEntryId,
    payload: &'a str,
    priority: i64,
    not_before_ms: u64,
    fields: &'a [(&'a str, &'a str)],
}

impl<'a> ValkeyStructuredQueueEntry<'a> {
    /// Creates a queue entry from a typed id and serialized payload.
    #[must_use]
    pub const fn new(entry_id: ValkeyQueueEntryId, payload: &'a str) -> Self {
        Self {
            entry_id,
            payload,
            priority: 0,
            not_before_ms: 0,
            fields: &[],
        }
    }

    /// Sets the queue priority.
    #[must_use]
    pub const fn with_priority(mut self, priority: i64) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the earliest claim timestamp.
    #[must_use]
    pub const fn with_not_before_ms(mut self, not_before_ms: u64) -> Self {
        self.not_before_ms = not_before_ms;
        self
    }

    /// Adds indexed fields used by claim filters.
    #[must_use]
    pub const fn with_fields(mut self, fields: &'a [(&'a str, &'a str)]) -> Self {
        self.fields = fields;
        self
    }

    pub(super) const fn entry_id(&self) -> &ValkeyQueueEntryId {
        &self.entry_id
    }

    pub(super) const fn payload(&self) -> &str {
        self.payload
    }

    pub(super) const fn priority(&self) -> i64 {
        self.priority
    }

    pub(super) const fn not_before_ms(&self) -> u64 {
        self.not_before_ms
    }

    pub(super) const fn fields(&self) -> &[(&'a str, &'a str)] {
        self.fields
    }
}

/// Exact field filter for a structured claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValkeyStructuredClaimFilter<'a> {
    field: &'a str,
    value: &'a str,
}

impl<'a> ValkeyStructuredClaimFilter<'a> {
    /// Creates an exact claim filter.
    #[must_use]
    pub const fn new(field: &'a str, value: &'a str) -> Self {
        Self { field, value }
    }

    pub(super) const fn field(&self) -> &str {
        self.field
    }

    pub(super) const fn value(&self) -> &str {
        self.value
    }
}

/// Structured claim request for one queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValkeyStructuredClaimRequest<'a> {
    worker_id: &'a ValkeyWorkerId,
    lease_id: &'a ValkeyLeaseId,
    now_ms: u64,
    lease_ttl_ms: u64,
    filters: &'a [ValkeyStructuredClaimFilter<'a>],
}

impl<'a> ValkeyStructuredClaimRequest<'a> {
    /// Creates a claim request.
    #[must_use]
    pub const fn new(
        worker_id: &'a ValkeyWorkerId,
        lease_id: &'a ValkeyLeaseId,
        now_ms: u64,
        lease_ttl_ms: u64,
    ) -> Self {
        Self {
            worker_id,
            lease_id,
            now_ms,
            lease_ttl_ms,
            filters: &[],
        }
    }

    /// Adds exact field filters to the request.
    #[must_use]
    pub const fn with_filters(mut self, filters: &'a [ValkeyStructuredClaimFilter<'a>]) -> Self {
        self.filters = filters;
        self
    }

    pub(super) const fn worker_id(&self) -> &ValkeyWorkerId {
        self.worker_id
    }

    pub(super) const fn lease_id(&self) -> &ValkeyLeaseId {
        self.lease_id
    }

    pub(super) const fn now_ms(&self) -> u64 {
        self.now_ms
    }

    pub(super) const fn lease_ttl_ms(&self) -> u64 {
        self.lease_ttl_ms
    }

    pub(super) const fn filters(&self) -> &[ValkeyStructuredClaimFilter<'a>] {
        self.filters
    }
}

/// A structured queue lease returned by claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValkeyStructuredQueueLease {
    lease_id: ValkeyLeaseId,
    entry_id: ValkeyQueueEntryId,
    payload: String,
    worker_id: ValkeyWorkerId,
    acquired_at_ms: u64,
    expires_at_ms: u64,
}

impl ValkeyStructuredQueueLease {
    pub(super) const fn new(
        lease_id: ValkeyLeaseId,
        entry_id: ValkeyQueueEntryId,
        payload: String,
        worker_id: ValkeyWorkerId,
        acquired_at_ms: u64,
        expires_at_ms: u64,
    ) -> Self {
        Self {
            lease_id,
            entry_id,
            payload,
            worker_id,
            acquired_at_ms,
            expires_at_ms,
        }
    }

    /// Borrows the lease id.
    #[must_use]
    pub const fn lease_id(&self) -> &ValkeyLeaseId {
        &self.lease_id
    }

    /// Borrows the claimed entry id.
    #[must_use]
    pub const fn entry_id(&self) -> &ValkeyQueueEntryId {
        &self.entry_id
    }

    /// Borrows the serialized typed payload.
    #[must_use]
    pub fn payload(&self) -> &str {
        self.payload.as_str()
    }

    /// Borrows the worker id.
    #[must_use]
    pub const fn worker_id(&self) -> &ValkeyWorkerId {
        &self.worker_id
    }

    /// Returns the claim timestamp.
    #[must_use]
    pub const fn acquired_at_ms(&self) -> u64 {
        self.acquired_at_ms
    }

    /// Returns the lease expiry timestamp.
    #[must_use]
    pub const fn expires_at_ms(&self) -> u64 {
        self.expires_at_ms
    }
}

/// Borrowed reference to a queue lease.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValkeyStructuredQueueLeaseRef<'a> {
    lease: &'a ValkeyLeaseId,
    entry: &'a ValkeyQueueEntryId,
    worker: &'a ValkeyWorkerId,
}

impl<'a> ValkeyStructuredQueueLeaseRef<'a> {
    /// Creates a typed lease reference.
    #[must_use]
    pub const fn new(
        lease_id: &'a ValkeyLeaseId,
        entry_id: &'a ValkeyQueueEntryId,
        worker_id: &'a ValkeyWorkerId,
    ) -> Self {
        Self {
            lease: lease_id,
            entry: entry_id,
            worker: worker_id,
        }
    }

    pub(super) const fn lease_id(&self) -> &ValkeyLeaseId {
        self.lease
    }

    pub(super) const fn entry_id(&self) -> &ValkeyQueueEntryId {
        self.entry
    }

    pub(super) const fn worker_id(&self) -> &ValkeyWorkerId {
        self.worker
    }
}

/// Result of a lease script.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValkeyLeaseScriptResult {
    /// Lease mutation was applied.
    Applied,
    /// Lease was missing or not eligible.
    NotApplied,
}

pub(super) fn priority_score(priority: i64) -> i64 {
    priority.saturating_neg()
}

pub(super) fn lease_not_owned_error(
    lease_id: ValkeyLeaseId,
    worker_id: ValkeyWorkerId,
) -> ValkeyStoreError {
    ValkeyStoreError::LeaseNotOwned {
        ownership: ValkeyLeaseOwnership::new(lease_id, worker_id),
    }
}
