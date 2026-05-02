#[path = "build/mod.rs"]
mod build;
#[path = "query/mod.rs"]
mod query;
mod schema;

#[cfg(test)]
pub(crate) use build::ensure_attachment_index_started;
pub(crate) use build::ensure_attachment_index_started_with_scanned_files;
#[cfg(test)]
pub(crate) use build::plan_attachment_build;
#[cfg(test)]
pub(crate) use build::{AttachmentBuildError, publish_attachments_from_projects};
pub(crate) use query::{AttachmentSearchError, search_attachment_hits};
