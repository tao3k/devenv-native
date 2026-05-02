//! Lightweight image attachment preflight audit helpers.

mod core;
mod dimensions;
mod format;
mod routing;

pub use core::{AttachmentAudit, audit_image_attachment, is_supported_image_path};

#[cfg(test)]
#[path = "../../tests/unit/image_audit.rs"]
mod tests;
