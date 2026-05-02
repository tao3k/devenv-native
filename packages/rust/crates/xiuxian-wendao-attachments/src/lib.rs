//! Attachment parsing, audit, and artifact helpers for Wendao document surfaces.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!();

#[cfg(feature = "archive-audit")]
/// Archive attachment manifest auditing for routing and cache planning.
pub mod archive_audit;

/// Image attachment preflight auditing for routing and cache planning.
pub mod image_audit;

#[doc(hidden)]
pub mod pdf;
