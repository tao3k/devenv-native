//! Lightweight PDF page complexity profile for source-range OCR planning.

use std::path::Path;

use lopdf::{Document as LopdfDocument, ObjectId, content::Operation};

/// Lightweight facts derived from one source PDF page content stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfSourcePageProfile {
    /// Zero-based page index.
    pub page_index: u32,
    /// Decoded content stream byte count.
    pub content_bytes: u32,
    /// PDF content operation count.
    pub operation_count: u32,
    /// Text-showing operation count.
    pub text_show_ops: u32,
    /// Path drawing operation count.
    pub path_ops: u32,
    /// Rectangle path operation count.
    pub rectangle_ops: u32,
    /// External object draw operation count.
    pub draw_object_ops: u32,
    /// Conservative planner weight derived from the operation counts.
    pub estimated_weight: u32,
}

/// Return page-level source PDF complexity profiles.
///
/// # Errors
///
/// Returns an error if the PDF cannot be loaded or page content streams cannot
/// be decoded.
pub fn source_pdf_page_profiles(path: &Path) -> Result<Vec<PdfSourcePageProfile>, String> {
    let document =
        LopdfDocument::load(path).map_err(|error| format!("load PDF with lopdf: {error}"))?;
    document
        .get_pages()
        .into_values()
        .enumerate()
        .map(|(page_index, page_id)| {
            source_pdf_page_profile(
                &document,
                u32::try_from(page_index).unwrap_or(u32::MAX),
                page_id,
            )
        })
        .collect()
}

fn source_pdf_page_profile(
    document: &LopdfDocument,
    page_index: u32,
    page_id: ObjectId,
) -> Result<PdfSourcePageProfile, String> {
    let content_bytes = document
        .get_page_content(page_id)
        .map_err(|error| format!("read PDF page {page_index} content: {error}"))?;
    let content = lopdf::content::Content::decode(content_bytes.as_slice())
        .map_err(|error| format!("decode PDF page {page_index} content: {error}"))?;
    let mut profile = PdfSourcePageProfile {
        page_index,
        content_bytes: u32::try_from(content_bytes.len()).unwrap_or(u32::MAX),
        operation_count: u32::try_from(content.operations.len()).unwrap_or(u32::MAX),
        text_show_ops: 0,
        path_ops: 0,
        rectangle_ops: 0,
        draw_object_ops: 0,
        estimated_weight: 1,
    };
    for operation in &content.operations {
        update_profile_counts(&mut profile, operation);
    }
    profile.estimated_weight = estimated_weight(&profile);
    Ok(profile)
}

fn update_profile_counts(profile: &mut PdfSourcePageProfile, operation: &Operation) {
    match operation.operator.as_str() {
        "Tj" | "TJ" | "'" | "\"" => {
            profile.text_show_ops = profile.text_show_ops.saturating_add(1);
        }
        "Do" => {
            profile.draw_object_ops = profile.draw_object_ops.saturating_add(1);
        }
        "re" => {
            profile.rectangle_ops = profile.rectangle_ops.saturating_add(1);
            profile.path_ops = profile.path_ops.saturating_add(1);
        }
        "m" | "l" | "c" | "v" | "y" | "h" | "S" | "s" | "f" | "F" | "f*" | "B" | "B*" | "b"
        | "b*" | "n" | "W" | "W*" => {
            profile.path_ops = profile.path_ops.saturating_add(1);
        }
        _ => {}
    }
}

fn estimated_weight(profile: &PdfSourcePageProfile) -> u32 {
    1_u32
        .saturating_add(profile.text_show_ops)
        .saturating_add(profile.operation_count.div_ceil(24))
        .saturating_add(profile.content_bytes.div_ceil(4096))
        .saturating_add(profile.path_ops.div_ceil(4))
        .saturating_add(profile.rectangle_ops.saturating_mul(4))
        .saturating_add(profile.draw_object_ops.saturating_mul(6))
}

#[cfg(test)]
#[path = "../../tests/unit/pdf/profile.rs"]
mod tests;
