//! `search::attachment::build` owns Wendao search attachment build behavior.

mod extract;
mod orchestration;
mod plan;
mod types;
mod write;

#[cfg(test)]
#[path = "../../../../tests/unit/search/attachment/build/mod.rs"]
mod tests;

pub(crate) use extract::attachment_kind_label;
#[cfg(any(test, feature = "test-support"))]
pub(crate) use orchestration::ensure_attachment_index_started;
pub(crate) use orchestration::ensure_attachment_index_started_with_scanned_files;
#[cfg(any(test, feature = "test-support"))]
pub(crate) use orchestration::publish_attachments_from_projects;
#[cfg(any(test, feature = "test-support"))]
pub(crate) use plan::plan_attachment_build;
pub(crate) use plan::plan_attachment_build_with_scanned_files;
#[cfg(any(test, feature = "test-support"))]
pub use types::AttachmentBuildError;
pub(crate) use types::{AttachmentBuildPlan, AttachmentWriteResult};
pub(crate) use write::write_attachment_epoch;
