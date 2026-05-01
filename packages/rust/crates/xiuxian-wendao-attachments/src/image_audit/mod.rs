//! Lightweight image attachment preflight audit helpers.

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use serde::Serialize;

mod dimensions;
mod format;
mod routing;

use dimensions::parse_dimensions;
use format::image_format_hint;
use routing::image_routing_decision;

const HEADER_READ_LIMIT_BYTES: u64 = 1024 * 1024;

/// Bounded Rust-side metadata for one image attachment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentAudit {
    /// Source path that was audited.
    pub source_path: String,
    /// File size from local filesystem metadata.
    pub file_size_bytes: u64,
    /// Normalized image format hint.
    pub format: String,
    /// MIME type derived from the file suffix.
    pub mime_type: String,
    /// Pixel width when the bounded header probe could prove it.
    pub width_px: Option<u32>,
    /// Pixel height when the bounded header probe could prove it.
    pub height_px: Option<u32>,
    /// Width multiplied by height when dimensions are known.
    pub pixel_count: Option<u64>,
    /// Header source used for dimensions, or the reason dimensions are absent.
    pub dimension_source: String,
    /// Audit-only Rust acceleration candidate.
    pub rust_acceleration_candidate: String,
    /// Human-readable reason for the candidate.
    pub decision_reason: String,
}

/// Return whether the file suffix is a known image suffix used by Docling.
#[must_use]
pub fn is_supported_image_path(path: &Path) -> bool {
    image_format_hint(path).is_some()
}

/// Audit a local image attachment for Rust-side routing and cache planning.
///
/// # Errors
///
/// Returns an error when file metadata or the bounded header read fails.
pub fn audit_image_attachment(path: &Path) -> Result<AttachmentAudit, String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("stat image attachment `{}`: {error}", path.display()))?;
    let Some(format_hint) = image_format_hint(path) else {
        return Ok(AttachmentAudit {
            source_path: path.to_string_lossy().to_string(),
            file_size_bytes: metadata.len(),
            format: "unknown".to_string(),
            mime_type: "application/octet-stream".to_string(),
            width_px: None,
            height_px: None,
            pixel_count: None,
            dimension_source: "unsupported".to_string(),
            rust_acceleration_candidate: "unsupported_non_image".to_string(),
            decision_reason: "source suffix is not a known Docling image format".to_string(),
        });
    };

    let header = read_header(path)?;
    let dimensions = parse_dimensions(format_hint.format, header.as_slice());
    let pixel_count = dimensions.map(|(width, height, _)| u64::from(width) * u64::from(height));
    let (width_px, height_px, dimension_source) = dimensions
        .map_or((None, None, "suffix_only"), |(width, height, source)| {
            (Some(width), Some(height), source)
        });
    let (candidate, reason) = image_routing_decision(metadata.len(), pixel_count);

    Ok(AttachmentAudit {
        source_path: path.to_string_lossy().to_string(),
        file_size_bytes: metadata.len(),
        format: format_hint.format.to_string(),
        mime_type: format_hint.mime_type.to_string(),
        width_px,
        height_px,
        pixel_count,
        dimension_source: dimension_source.to_string(),
        rust_acceleration_candidate: candidate.to_string(),
        decision_reason: reason.to_string(),
    })
}

fn read_header(path: &Path) -> Result<Vec<u8>, String> {
    let file = File::open(path)
        .map_err(|error| format!("open image attachment `{}`: {error}", path.display()))?;
    let mut limited = file.take(HEADER_READ_LIMIT_BYTES);
    let mut bytes = Vec::new();
    limited
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read image attachment header `{}`: {error}", path.display()))?;
    Ok(bytes)
}

#[cfg(test)]
#[path = "../../tests/unit/image_audit.rs"]
mod tests;
