//! Lightweight image attachment preflight audit helpers.

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use serde::Serialize;

const HEADER_READ_LIMIT_BYTES: u64 = 1024 * 1024;
const LARGE_IMAGE_BYTES_THRESHOLD: u64 = 20 * 1024 * 1024;
const LARGE_IMAGE_PIXEL_THRESHOLD: u64 = 25_000_000;

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

#[derive(Debug, Clone, Copy)]
struct ImageFormatHint {
    format: &'static str,
    mime_type: &'static str,
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

fn image_format_hint(path: &Path) -> Option<ImageFormatHint> {
    let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();
    match extension.as_str() {
        "png" => Some(ImageFormatHint {
            format: "png",
            mime_type: "image/png",
        }),
        "jpg" | "jpeg" => Some(ImageFormatHint {
            format: "jpeg",
            mime_type: "image/jpeg",
        }),
        "tif" | "tiff" => Some(ImageFormatHint {
            format: "tiff",
            mime_type: "image/tiff",
        }),
        "bmp" => Some(ImageFormatHint {
            format: "bmp",
            mime_type: "image/bmp",
        }),
        "webp" => Some(ImageFormatHint {
            format: "webp",
            mime_type: "image/webp",
        }),
        "gif" => Some(ImageFormatHint {
            format: "gif",
            mime_type: "image/gif",
        }),
        _ => None,
    }
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

fn parse_dimensions(format: &str, bytes: &[u8]) -> Option<(u32, u32, &'static str)> {
    match format {
        "png" => png_dimensions(bytes).map(|(width, height)| (width, height, "png_ihdr")),
        "jpeg" => jpeg_dimensions(bytes).map(|(width, height)| (width, height, "jpeg_sof")),
        "bmp" => bmp_dimensions(bytes).map(|(width, height)| (width, height, "bmp_dib")),
        _ => None,
    }
}

fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 24 || &bytes[0..8] != PNG_SIGNATURE || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    non_zero_dimensions(width, height)
}

fn jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 4 || bytes[0] != 0xff || bytes[1] != 0xd8 {
        return None;
    }
    let mut index = 2usize;
    while index + 3 < bytes.len() {
        while index < bytes.len() && bytes[index] != 0xff {
            index += 1;
        }
        while index < bytes.len() && bytes[index] == 0xff {
            index += 1;
        }
        if index >= bytes.len() {
            return None;
        }
        let marker = bytes[index];
        index += 1;
        if marker == 0xd9 || marker == 0xda {
            return None;
        }
        if marker == 0x01 || (0xd0..=0xd7).contains(&marker) {
            continue;
        }
        if index + 1 >= bytes.len() {
            return None;
        }
        let segment_len = u16::from_be_bytes([bytes[index], bytes[index + 1]]) as usize;
        if segment_len < 2 {
            return None;
        }
        let segment_start = index + 2;
        let segment_end = index + segment_len;
        if segment_end > bytes.len() {
            return None;
        }
        if is_jpeg_start_of_frame(marker) && segment_start + 4 < segment_end {
            let height = u16::from_be_bytes([bytes[segment_start + 1], bytes[segment_start + 2]]);
            let width = u16::from_be_bytes([bytes[segment_start + 3], bytes[segment_start + 4]]);
            return non_zero_dimensions(u32::from(width), u32::from(height));
        }
        index = segment_end;
    }
    None
}

fn is_jpeg_start_of_frame(marker: u8) -> bool {
    matches!(
        marker,
        0xc0 | 0xc1 | 0xc2 | 0xc3 | 0xc5 | 0xc6 | 0xc7 | 0xc9 | 0xca | 0xcb | 0xcd | 0xce | 0xcf
    )
}

fn bmp_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 26 || &bytes[0..2] != b"BM" {
        return None;
    }
    let width = i32::from_le_bytes(bytes[18..22].try_into().ok()?);
    let height = i32::from_le_bytes(bytes[22..26].try_into().ok()?);
    if width <= 0 || height == 0 {
        return None;
    }
    non_zero_dimensions(u32::try_from(width).ok()?, height.unsigned_abs())
}

fn non_zero_dimensions(width: u32, height: u32) -> Option<(u32, u32)> {
    (width > 0 && height > 0).then_some((width, height))
}

fn image_routing_decision(
    file_size_bytes: u64,
    pixel_count: Option<u64>,
) -> (&'static str, &'static str) {
    if pixel_count.is_none() {
        return (
            "docling_passthrough",
            "Rust recognized the image format but did not prove dimensions from the header",
        );
    }
    if file_size_bytes >= LARGE_IMAGE_BYTES_THRESHOLD
        || pixel_count.is_some_and(|pixels| pixels >= LARGE_IMAGE_PIXEL_THRESHOLD)
    {
        return (
            "oversized_image_preflight_candidate",
            "Rust can preflight size before a future crop or tile OCR strategy",
        );
    }
    (
        "image_ocr_cache_candidate",
        "Rust can key future whole-image OCR cache and preserve Docling as OCR authority",
    )
}

#[cfg(test)]
#[path = "../tests/unit/image_audit.rs"]
mod tests;
