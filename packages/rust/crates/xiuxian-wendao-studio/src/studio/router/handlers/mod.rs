//! Studio API endpoint handlers.

#[path = "analysis/mod.rs"]
pub(crate) mod analysis;
#[path = "capabilities/mod.rs"]
pub(crate) mod capabilities;
/// Docs-facing deep-wiki planning handlers.
#[path = "docs/mod.rs"]
pub(crate) mod docs;
#[path = "document_extract_jobs.rs"]
pub(crate) mod document_extract_jobs;
#[path = "document_extract_resource.rs"]
pub(crate) mod document_extract_resource;
#[path = "document_extract_result.rs"]
pub(crate) mod document_extract_result;
#[path = "episteme/mod.rs"]
pub(crate) mod episteme;
#[path = "graph/mod.rs"]
pub(crate) mod graph;
#[path = "model_route.rs"]
pub(crate) mod model_route;
#[path = "repo/mod.rs"]
pub(crate) mod repo;
#[path = "vfs.rs"]
pub(crate) mod vfs;
