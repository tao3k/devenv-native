#[path = "build/mod.rs"]
mod build;
#[path = "query/mod.rs"]
mod query;
mod schema;

#[cfg(any(test, feature = "test-support"))]
pub use build::AttachmentBuildError;
#[cfg(any(test, feature = "test-support"))]
pub(crate) use build::ensure_attachment_index_started;
pub(crate) use build::ensure_attachment_index_started_with_scanned_files;
#[cfg(test)]
pub(crate) use build::plan_attachment_build;
#[cfg(any(test, feature = "test-support"))]
pub(crate) use build::publish_attachments_from_projects;
pub use query::AttachmentSearchError;
pub(crate) use query::search_attachment_hits;
