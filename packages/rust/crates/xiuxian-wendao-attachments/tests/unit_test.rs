//! Cargo entry point for `xiuxian-wendao-attachments` unit tests.

#[path = "unit/lib_policy.rs"]
mod lib_policy;

#[path = "unit/audio/mod.rs"]
mod audio;

#[cfg(feature = "legacy-office")]
#[path = "unit/legacy_office.rs"]
mod legacy_office;

#[cfg(feature = "legacy-office")]
#[path = "unit/legacy_office_markdown.rs"]
mod legacy_office_markdown;

#[cfg(feature = "legacy-office")]
#[path = "unit/legacy_office_metrics.rs"]
mod legacy_office_metrics;

#[cfg(feature = "legacy-office")]
#[path = "unit/legacy_office_xls.rs"]
mod legacy_office_xls;
