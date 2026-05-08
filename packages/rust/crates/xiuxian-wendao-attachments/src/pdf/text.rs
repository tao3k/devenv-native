//! Source PDF text extraction helpers for text-only OCR recovery paths.

use std::path::Path;

use lopdf::Document as LopdfDocument;

/// Extract text for the requested zero-based page indexes from a source PDF.
///
/// # Errors
///
/// Returns an error when the PDF cannot be loaded, a page index cannot be
/// converted to a PDF page number, or lopdf fails to extract text for a page.
pub fn source_pdf_page_texts(path: &Path, page_indexes: &[u32]) -> Result<Vec<String>, String> {
    let document =
        LopdfDocument::load(path).map_err(|error| format!("load PDF with lopdf: {error}"))?;
    page_indexes
        .iter()
        .copied()
        .map(|page_index| source_pdf_page_text(&document, page_index))
        .collect()
}

fn source_pdf_page_text(document: &LopdfDocument, page_index: u32) -> Result<String, String> {
    let page_number = page_index
        .checked_add(1)
        .ok_or_else(|| format!("source PDF page index {page_index} overflowed page number"))?;
    document
        .extract_text(&[page_number])
        .map(|text| text.trim().to_string())
        .map_err(|error| format!("extract source PDF page {page_index} text with lopdf: {error}"))
}

#[cfg(test)]
#[path = "../../tests/unit/pdf/text.rs"]
mod tests;
