//! Project Sentinel: Real-time synchronization and semantic change propagation.
//!
//! This module provides the infrastructure for observing the filesystem and
//! automatically updating the `LinkGraph` and Audit reports when files change.
//!
//! ## Phase 6: Semantic Change Propagation
//!
//! When source code changes, Sentinel identifies "Observational Casualties" -
//! documents with `:OBSERVE:` patterns that may reference the changed code.
//! These are surfaced as `SemanticDriftSignal` events for agent notification.

mod analysis;
mod filters;
mod observations;
mod types;
mod watch;

#[cfg(test)]
pub(crate) use std::path::Path;
#[cfg(test)]
pub(crate) use tokio::sync::mpsc;

#[cfg(test)]
pub(crate) use self::analysis::{matches_scope_filter, to_pascal_case};
#[cfg(test)]
pub(crate) use self::filters::{
    is_high_noise_file, is_ignorable_path, is_source_code, verify_file_stable,
};
pub use analysis::{compute_file_hash, extract_pattern_symbols, propagate_source_change};
pub use observations::{
    ObservationBus, ObservationRef, ObservationSignal, signals_to_status_batch,
};
pub use types::{AffectedDoc, DriftConfidence, SemanticDriftSignal};
pub use watch::{Sentinel, SentinelConfig};

#[cfg(test)]
#[path = "../../../../tests/unit/zhenfa_router/native/sentinel/mod.rs"]
mod tests;
