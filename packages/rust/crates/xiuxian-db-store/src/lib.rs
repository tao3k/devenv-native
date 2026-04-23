//! Storage-backed database surface for `xiuxian-vector` and lightweight
//! client-local persistence helpers.
//!
//! This crate is the explicit dependency boundary for storage concerns that
//! should not leak into all callers:
//! - the heavy Lance-backed `vector-store` surface stays feature-gated
//! - the lightweight local `SQLite` surface stays feature-gated
//!
//! Lightweight Arrow/DataFusion helpers stay in `xiuxian-vector`; storage-bound
//! callers should depend on this crate instead of widening their dependency
//! graph directly.

#[cfg(feature = "sqlite")]
pub mod sql;

#[cfg(feature = "sqlite")]
pub use rusqlite;

#[cfg(feature = "vector-store")]
pub use xiuxian_vector::*;
