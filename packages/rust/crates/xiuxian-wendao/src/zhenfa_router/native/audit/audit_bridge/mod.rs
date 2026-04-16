//! Audit bridge for batch fixes and surgical repair.

mod batch_fix;
mod bridge;
mod helpers;
mod surgical;
mod types;

#[allow(unused_imports)]
pub use bridge::{AuditBridge, DefaultAuditBridge, generate_batch_fixes};
pub(crate) use helpers::{compute_hash, resolve_file_content};
pub use surgical::generate_surgical_fixes;
pub use types::{BatchFix, BatchFixMode, ByteRange, FixResult};

#[cfg(test)]
#[path = "../../../../../tests/unit/zhenfa_router/native/audit/audit_bridge/mod.rs"]
mod tests;
