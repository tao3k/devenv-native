//! OCR result document-resource Arrow batch builder.

use arrow::record_batch::RecordBatch;

use super::support::{
    document_resource_contract, record_batch, result_int_column, result_string_column,
};
use crate::pdf::ocr::types::{PdfOcrShardResult, PdfOcrShardResultStatus};

/// # Errors
///
/// Returns an error if Arrow cannot build the stable document-resource batch.
pub fn build_ocr_result_resource_batch(
    results: &[PdfOcrShardResult],
) -> Result<RecordBatch, String> {
    record_batch(
        &document_resource_contract(),
        vec![
            result_string_column(results, |result| result.source_path.clone()),
            result_string_column(results, |result| resource_type(result).to_string()),
            result_string_column(results, |result| result.image_path.clone()),
            result_int_column(results, |result| result.page_index),
            result_string_column(results, |result| {
                format!("OCR PDF page {}", result.page_index + 1)
            }),
            result_string_column(results, resource_content),
            result_string_column(results, |result| resource_mime_type(result).to_string()),
            result_string_column(results, |result| result.status.as_str().to_string()),
            result_string_column(results, |result| result.element_id.clone()),
        ],
        "build OCR result resource Arrow batch",
    )
}

fn resource_type(result: &PdfOcrShardResult) -> &'static str {
    match result.status {
        PdfOcrShardResultStatus::Succeeded => "ocr_text",
        PdfOcrShardResultStatus::Failed => "ocr_error",
        PdfOcrShardResultStatus::Skipped => "ocr_skipped",
    }
}

fn resource_mime_type(result: &PdfOcrShardResult) -> &str {
    match result.status {
        PdfOcrShardResultStatus::Succeeded => result.text_mime_type.as_str(),
        PdfOcrShardResultStatus::Failed | PdfOcrShardResultStatus::Skipped => "text/plain",
    }
}

fn resource_content(result: &PdfOcrShardResult) -> String {
    match result.status {
        PdfOcrShardResultStatus::Succeeded => result.text.clone().unwrap_or_default(),
        PdfOcrShardResultStatus::Failed | PdfOcrShardResultStatus::Skipped => {
            result.error_message.clone().unwrap_or_default()
        }
    }
}
