use std::collections::{HashMap, HashSet};

use xiuxian_wendao_attachments::pdf::ocr::{
    PdfOcrShardInput, PdfOcrShardResult, PdfOcrShardResultStatus,
};

pub(crate) fn validate_successful_ocr_results(
    results: &[PdfOcrShardResult],
    page_count: u32,
    shard_count: u32,
) -> Result<(), String> {
    if results.len() != usize::try_from(shard_count).unwrap_or(usize::MAX) {
        return Err(format!(
            "OCR worker returned {} rows for {shard_count} rendered shards",
            results.len()
        ));
    }
    for result in results {
        if result.page_index >= page_count {
            return Err(format!(
                "OCR worker returned out-of-range page {} for {page_count} page PDF",
                result.page_index
            ));
        }
        if result.status != PdfOcrShardResultStatus::Succeeded {
            return Err(format!(
                "OCR worker returned non-success status `{}` for page {}",
                result.status.as_str(),
                result.page_index
            ));
        }
        if result
            .text
            .as_deref()
            .is_none_or(|text| text.trim().is_empty())
        {
            return Err(format!(
                "OCR worker returned empty text for shard `{}`",
                result.shard_element_id
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_ocr_results_match_inputs(
    inputs: &[PdfOcrShardInput],
    results: &[PdfOcrShardResult],
) -> Result<(), String> {
    if inputs.len() != results.len() {
        return Err(format!(
            "OCR worker returned {} rows for {} inputs",
            results.len(),
            inputs.len()
        ));
    }
    let mut inputs_by_shard = HashMap::new();
    for input in inputs {
        if inputs_by_shard
            .insert(input.shard_element_id.as_str(), input)
            .is_some()
        {
            return Err(format!(
                "duplicate OCR shard input id `{}`",
                input.shard_element_id
            ));
        }
    }
    let mut result_shards = HashSet::new();
    for result in results {
        if !result_shards.insert(result.shard_element_id.as_str()) {
            return Err(format!(
                "duplicate OCR shard result id `{}`",
                result.shard_element_id
            ));
        }
        let input = inputs_by_shard
            .get(result.shard_element_id.as_str())
            .ok_or_else(|| {
                format!(
                    "OCR worker returned unknown shard id `{}`",
                    result.shard_element_id
                )
            })?;
        if input.page_index != result.page_index {
            return Err(format!(
                "OCR worker returned page {} for shard `{}` but input page was {}",
                result.page_index, result.shard_element_id, input.page_index
            ));
        }
        if input.raster_sha256 != result.raster_sha256 {
            return Err(format!(
                "OCR worker returned raster hash `{}` for shard `{}` but input hash was `{}`",
                result.raster_sha256, result.shard_element_id, input.raster_sha256
            ));
        }
    }
    Ok(())
}
