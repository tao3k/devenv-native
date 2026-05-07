//! OCR shard input/result contracts and Arrow batch conversion helpers.

mod batches;
mod recovery;
mod types;

pub use batches::{
    build_ocr_result_resource_batch, build_ocr_shard_input_batch, build_ocr_shard_inputs,
    build_ocr_shard_result_batch, decode_ocr_shard_input_batch, decode_ocr_shard_input_batches,
    decode_ocr_shard_result_batch, decode_ocr_shard_result_batches,
};
pub use recovery::{
    downgrade_hosted_vlm_region_parent_page_inputs, hosted_vlm_region_parent_page_shards,
    merge_hosted_vlm_recovery_region_inputs, prepare_hosted_vlm_recovery_region_inputs,
};
pub use types::{
    OcrShardManifestSource, PDF_OCR_DEFAULT_PROFILE, PDF_OCR_DOCLING_VLM_DEEPSEEK_OCR_PROFILE,
    PDF_OCR_FAST_TEXT_PROFILE, PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
    PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PDF_OCR_SHARD_RESULT_SCHEMA_VERSION, PdfOcrShardInput,
    PdfOcrShardResult, PdfOcrShardResultStatus, PdfOcrWorkerProfile, is_hosted_vlm_direct_profile,
};

#[cfg(test)]
#[path = "../../../tests/unit/pdf/ocr/mod.rs"]
mod tests;
