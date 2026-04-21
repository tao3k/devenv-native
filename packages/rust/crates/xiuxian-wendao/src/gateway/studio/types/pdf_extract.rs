use serde::{Deserialize, Serialize};
use specta::Type;

/// PDF extraction result returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PdfExtractResult {
    /// Source PDF path.
    pub source_path: String,
    /// Total number of pages in the PDF.
    pub total_pages: usize,
    /// Unix timestamp when extraction completed (from marker file).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extracted_at: Option<i64>,
    /// Extracted structured resources.
    pub resources: Vec<PdfExtractResource>,
}

/// One extracted resource from a PDF (paragraph, image, table, formula).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PdfExtractResource {
    /// Resource type: "document" | "image" | "table" | "formula".
    pub resource_type: String,
    /// VFS path to the extracted file (empty for inline text).
    pub resource_path: String,
    /// Page index (0-based).
    pub page_index: usize,
    /// Caption or title.
    pub caption: String,
    /// Text / HTML / LaTeX content.
    pub content: String,
    /// MIME type.
    pub mime_type: String,
    /// Extraction status: "ok" | "error" | "skipped".
    pub status: String,
    /// Element ID from the extractor.
    pub element_id: String,
}
