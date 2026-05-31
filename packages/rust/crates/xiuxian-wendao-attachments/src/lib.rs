//! Attachment parsing, audit, and artifact helpers for Wendao document surfaces.

/// Model-agnostic audio shard planning and admission identity contracts.
pub mod audio;

#[cfg(feature = "archive-audit")]
/// Archive attachment manifest auditing for routing and cache planning.
pub mod archive_audit;

/// Image attachment preflight auditing for routing and cache planning.
pub mod image_audit;

#[cfg(feature = "image-shards")]
/// Standalone image attachment shard planning and materialization.
pub mod image_shards;

#[cfg(feature = "legacy-office")]
/// Legacy Microsoft Office attachment extraction through the Rust parser stack.
pub mod legacy_office;

#[doc(hidden)]
pub mod pdf;

/// Read-only projections from attachment-owned contracts into polyglot contracts.
pub mod polyglot;
