//! Archive attachment preflight audit helpers.

mod audit;
mod format;
mod member;
mod routing;
mod types;

pub use audit::{audit_archive_attachment, is_supported_archive_path};
pub use types::{ArchiveAttachmentAudit, ArchiveMemberAudit};

#[cfg(test)]
#[path = "../../tests/unit/archive_audit.rs"]
mod tests;
