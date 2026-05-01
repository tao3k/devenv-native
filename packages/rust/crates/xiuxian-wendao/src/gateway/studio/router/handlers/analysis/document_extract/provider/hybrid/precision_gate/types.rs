use arrow::record_batch::RecordBatch as EngineRecordBatch;
use xiuxian_wendao_attachments::pdf::ocr::{PdfOcrShardInput, PdfOcrShardResult};
use xiuxian_wendao_attachments::pdf::structure::DocumentStructureBlock;

pub(super) struct HybridPrecisionGateInput<'a> {
    pub(super) page_count: u32,
    pub(super) text_page_indices: &'a [u32],
    pub(super) resource_batch: &'a EngineRecordBatch,
    pub(super) structure_blocks: &'a [DocumentStructureBlock],
    pub(super) ocr_inputs: &'a [PdfOcrShardInput],
    pub(super) ocr_results: &'a [PdfOcrShardResult],
}
