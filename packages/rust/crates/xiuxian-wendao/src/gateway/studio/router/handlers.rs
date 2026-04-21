//! Studio API endpoint handlers.

#[path = "handlers/analysis/mod.rs"]
pub(crate) mod analysis;
#[path = "handlers/capabilities/mod.rs"]
pub(crate) mod capabilities;
/// Docs-facing deep-wiki planning handlers.
#[path = "handlers/docs.rs"]
pub(crate) mod docs;
#[path = "handlers/graph.rs"]
pub(crate) mod graph;
#[path = "handlers/repo.rs"]
pub(crate) mod repo;
#[path = "handlers/vfs.rs"]
pub(crate) mod vfs;
#[path = "handlers/pdf_extract_result.rs"]
pub(crate) mod pdf_extract_result;
#[path = "handlers/pdf_extract_resource.rs"]
pub(crate) mod pdf_extract_resource;
