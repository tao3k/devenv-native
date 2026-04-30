use std::path::PathBuf;

use arrow::record_batch::RecordBatch as EngineRecordBatch;
use xiuxian_wendao_attachments::pdf::ocr::{PdfOcrShardInput, PdfOcrShardResult};
use xiuxian_wendao_attachments::pdf::render::PdfPageRegionRenderRequest;

pub(in super::super) const DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_SELECTION";
pub(in super::super) const DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_JSON";

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HybridPdfRegionInput {
    pub(super) source: PathBuf,
    pub(super) regions: Vec<PdfPageRegionRenderRequest>,
}

pub(in super::super) struct HybridDocumentResourceBatch {
    pub(super) batch: EngineRecordBatch,
    pub(super) ocr_inputs: Vec<PdfOcrShardInput>,
    pub(super) ocr_results: Vec<PdfOcrShardResult>,
}

impl HybridDocumentResourceBatch {
    #[cfg(test)]
    pub(in super::super) fn native(batch: EngineRecordBatch) -> Self {
        Self {
            batch,
            ocr_inputs: Vec::new(),
            ocr_results: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(in super::super) fn with_ocr(
        batch: EngineRecordBatch,
        ocr_inputs: Vec<PdfOcrShardInput>,
        ocr_results: Vec<PdfOcrShardResult>,
    ) -> Self {
        Self {
            batch,
            ocr_inputs,
            ocr_results,
        }
    }
}
