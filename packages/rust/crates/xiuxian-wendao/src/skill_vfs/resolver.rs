//! Skill VFS resolver core implementation and URI resolution.

/// Skill VFS resolver core implementation.
#[path = "resolver/core.rs"]
pub mod core;
/// Embedded mount helpers for semantic resources.
#[path = "resolver/mount.rs"]
mod mount;
/// Cached UTF-8 read helpers.
#[path = "resolver/read.rs"]
mod read;
/// URI resolution logic for skill VFS.
#[path = "resolver/resolve_uri.rs"]
pub mod resolve_uri;
/// Runtime discovery helpers for skill roots.
#[path = "resolver/runtime.rs"]
mod runtime;
