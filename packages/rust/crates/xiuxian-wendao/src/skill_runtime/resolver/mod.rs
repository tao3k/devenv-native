//! Skill runtime resolver core implementation and URI resolution.

/// Skill runtime resolver core implementation.
#[path = "core.rs"]
pub mod core;
/// Embedded mount helpers for semantic resources.
#[path = "mount.rs"]
mod mount;
/// Cached UTF-8 read helpers.
#[path = "read.rs"]
mod read;
/// URI resolution logic for skill runtime.
#[path = "resolve_uri.rs"]
pub mod resolve_uri;
/// Runtime discovery helpers for skill roots.
#[path = "runtime.rs"]
mod runtime;
