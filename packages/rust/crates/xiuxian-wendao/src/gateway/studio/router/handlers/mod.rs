//! Studio API endpoint handlers.

pub(crate) mod analysis;
pub(crate) mod capabilities;
/// Docs-facing deep-wiki planning handlers.
#[path = "docs/mod.rs"]
pub(crate) mod docs;
pub(crate) mod graph;
pub(crate) mod repo;
pub(crate) mod vfs;
