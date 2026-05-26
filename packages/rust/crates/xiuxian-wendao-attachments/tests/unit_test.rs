//! Cargo entry point for `xiuxian-wendao-attachments` unit tests.

#[path = "unit/lib_policy.rs"]
mod lib_policy;

#[path = "unit/audio/mod.rs"]
mod audio;

#[cfg(feature = "legacy-office")]
#[path = "unit/legacy_office.rs"]
mod legacy_office;
