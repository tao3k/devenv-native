use arrow::record_batch::RecordBatch as EngineRecordBatch;
use xiuxian_wendao_attachments::pdf::ocr::{PdfOcrShardInput, PdfOcrShardResult};
use xiuxian_wendao_attachments::pdf::structure::DocumentStructureBlock;

use super::coverage::{validate_hybrid_page_coverage, validate_hybrid_shard_coverage};
use super::ocr::validate_successful_ocr_results_for_inputs;
use super::resource::validate_resource_rows;
use super::structure::validate_structure_rows;
use super::types::HybridPrecisionGateInput;

pub(crate) fn validate_hybrid_precision_gate(
    page_count: u32,
    text_page_indices: &[u32],
    resource_batch: &EngineRecordBatch,
    structure_blocks: &[DocumentStructureBlock],
    ocr_inputs: &[PdfOcrShardInput],
    ocr_results: &[PdfOcrShardResult],
) -> Result<(), String> {
    let input = HybridPrecisionGateInput {
        page_count,
        text_page_indices,
        resource_batch,
        structure_blocks,
        ocr_inputs,
        ocr_results,
    };
    if input.ocr_inputs.is_empty() {
        validate_hybrid_page_coverage(input.page_count, input.text_page_indices, &[])?;
    } else {
        validate_successful_ocr_results_for_inputs(
            input.ocr_results,
            input.page_count,
            u32::try_from(input.ocr_inputs.len()).unwrap_or(u32::MAX),
            input.ocr_inputs,
        )?;
        validate_hybrid_shard_coverage(
            input.page_count,
            input.text_page_indices,
            input.ocr_inputs,
            input.ocr_results,
        )?;
    }
    validate_resource_rows(input.resource_batch, input.ocr_results.len())?;
    validate_structure_rows(
        input.page_count,
        input.structure_blocks,
        input.ocr_inputs,
        input.ocr_results,
    )
}
