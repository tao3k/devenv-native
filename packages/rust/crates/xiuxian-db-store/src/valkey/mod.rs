//! Structured Valkey primitives for hot queue indexes.
//!
//! This module owns Valkey command details for short-lived queue and lease
//! indexes. Domain crates provide typed JSON/Arrow payloads and explicit field
//! values; this layer never infers domain values by parsing composite keys.

mod client;
mod config;
mod error;
pub mod queue;

pub use client::ValkeyClient;
pub use config::{ValkeyKeyNamespace, ValkeyStoreConfig};
pub use error::{ValkeyLeaseOwnership, ValkeyStoreError};
pub use queue::{
    ValkeyLeaseId, ValkeyLeaseScriptResult, ValkeyQueueEntryId, ValkeyQueueKeys,
    ValkeyStructuredClaimFilter, ValkeyStructuredClaimRequest, ValkeyStructuredQueue,
    ValkeyStructuredQueueEntry, ValkeyStructuredQueueLease, ValkeyStructuredQueueLeaseRef,
    ValkeyWorkerId,
};
