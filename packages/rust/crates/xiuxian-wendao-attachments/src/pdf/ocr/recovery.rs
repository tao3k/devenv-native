//! OCR2 recovery shard binding helpers.

use std::collections::{BTreeMap, BTreeSet};

use super::types::{
    PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE, PDF_OCR_FAST_TEXT_PROFILE, PdfOcrShardInput,
};

/// Downgrade OCR2 page shards that are covered by OCR2 recovery regions to the
/// fast text profile so the page remains the deterministic parent surface.
pub fn downgrade_ocr2_region_parent_page_inputs(
    inputs: &mut [PdfOcrShardInput],
    region_pages: &BTreeSet<u32>,
) {
    for input in inputs {
        if input.shard_type == "page"
            && input.ocr_profile == PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE
            && region_pages.contains(&input.page_index)
        {
            input.ocr_profile = PDF_OCR_FAST_TEXT_PROFILE.to_string();
            input.ocr_engine = "docling-fast-text-ocr".to_string();
        }
    }
}

/// Return parent page shard ids keyed by page index.
#[must_use]
pub fn ocr2_region_parent_page_shards(inputs: &[PdfOcrShardInput]) -> BTreeMap<u32, String> {
    inputs
        .iter()
        .filter(|input| input.shard_type == "page")
        .map(|input| (input.page_index, input.shard_element_id.clone()))
        .collect()
}

/// Bind rendered OCR2 recovery region shards to parent page shards and stamp
/// the OCR2 direct VLM profile.
///
/// # Errors
///
/// Returns an error when the rendered input is not a region shard or no parent
/// page shard exists for the region page.
pub fn prepare_ocr2_recovery_region_inputs(
    parent_page_shards: &BTreeMap<u32, String>,
    rendered_inputs: Vec<PdfOcrShardInput>,
) -> Result<Vec<PdfOcrShardInput>, String> {
    rendered_inputs
        .into_iter()
        .map(|mut input| {
            if input.shard_type != "region" {
                return Err(format!(
                    "OCR2 recovery region render produced non-region shard `{}`",
                    input.shard_element_id
                ));
            }
            let parent_shard_element_id = parent_page_shards
                .get(&input.page_index)
                .ok_or_else(|| {
                    format!(
                        "OCR2 recovery region `{}` has no parent page shard for page {}",
                        input.shard_element_id, input.page_index
                    )
                })?
                .clone();
            input.parent_shard_element_id = parent_shard_element_id;
            input.ocr_profile = PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE.to_string();
            input.ocr_engine = "deepseek-ocr2-direct-vlm".to_string();
            Ok(input)
        })
        .collect()
}

/// Merge rendered OCR2 recovery regions into the original OCR shard inputs.
///
/// # Errors
///
/// Returns an error when rendered region inputs cannot be bound to parent page
/// shards.
pub fn merge_ocr2_recovery_region_inputs(
    mut inputs: Vec<PdfOcrShardInput>,
    rendered_inputs: Vec<PdfOcrShardInput>,
    region_pages: &BTreeSet<u32>,
) -> Result<Vec<PdfOcrShardInput>, String> {
    downgrade_ocr2_region_parent_page_inputs(&mut inputs, region_pages);
    let parent_page_shards = ocr2_region_parent_page_shards(inputs.as_slice());
    inputs.extend(prepare_ocr2_recovery_region_inputs(
        &parent_page_shards,
        rendered_inputs,
    )?);
    Ok(inputs)
}
