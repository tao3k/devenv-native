use std::collections::{HashMap, HashSet};

use xiuxian_wendao_attachments::pdf::ocr::{PdfOcrShardInput, PdfOcrShardResult};

pub(crate) fn order_ocr_results_by_inputs(
    inputs: &[PdfOcrShardInput],
    results: Vec<PdfOcrShardResult>,
) -> Result<Vec<PdfOcrShardResult>, String> {
    if inputs.len() != results.len() {
        return Err(format!(
            "OCR worker returned {} rows for {} inputs",
            results.len(),
            inputs.len()
        ));
    }

    validate_unique_input_shards(inputs)?;
    let mut results_by_shard = results_by_shard(results)?;
    let ordered = ordered_results_for_inputs(inputs, &mut results_by_shard)?;

    if let Some(unknown) = results_by_shard.keys().next() {
        return Err(format!("OCR worker returned unknown shard id `{unknown}`"));
    }

    Ok(ordered)
}

fn validate_unique_input_shards(inputs: &[PdfOcrShardInput]) -> Result<(), String> {
    inputs
        .iter()
        .try_fold(HashSet::new(), |mut input_shards, input| {
            if input_shards.insert(input.shard_element_id.as_str()) {
                Ok(input_shards)
            } else {
                Err(format!(
                    "duplicate OCR shard input id `{}`",
                    input.shard_element_id
                ))
            }
        })
        .map(|_| ())
}

fn results_by_shard(
    results: Vec<PdfOcrShardResult>,
) -> Result<HashMap<String, PdfOcrShardResult>, String> {
    results.into_iter().try_fold(
        HashMap::new(),
        |mut results_by_shard: HashMap<String, PdfOcrShardResult>, result| {
            let shard_id = result.shard_element_id.clone();
            if results_by_shard.insert(shard_id.clone(), result).is_none() {
                Ok(results_by_shard)
            } else {
                Err(format!("duplicate OCR shard result id `{shard_id}`"))
            }
        },
    )
}

fn ordered_results_for_inputs(
    inputs: &[PdfOcrShardInput],
    results_by_shard: &mut HashMap<String, PdfOcrShardResult>,
) -> Result<Vec<PdfOcrShardResult>, String> {
    inputs
        .iter()
        .map(|input| {
            let result = results_by_shard
                .remove(input.shard_element_id.as_str())
                .ok_or_else(|| {
                    format!(
                        "OCR worker did not return shard id `{}`",
                        input.shard_element_id
                    )
                })?;
            validate_ocr_result_matches_input(input, &result)?;
            Ok(result)
        })
        .collect()
}

pub(super) fn validate_ocr_result_matches_input(
    input: &PdfOcrShardInput,
    result: &PdfOcrShardResult,
) -> Result<(), String> {
    if input.shard_element_id != result.shard_element_id {
        return Err(format!(
            "OCR worker returned shard `{}` for input shard `{}`",
            result.shard_element_id, input.shard_element_id
        ));
    }
    if input.source_path != result.source_path {
        return Err(format!(
            "OCR worker returned source `{}` for shard `{}` but input source was `{}`",
            result.source_path, result.shard_element_id, input.source_path
        ));
    }
    if input.source_content_hash != result.source_content_hash {
        return Err(format!(
            "OCR worker returned source hash `{}` for shard `{}` but input hash was `{}`",
            result.source_content_hash, result.shard_element_id, input.source_content_hash
        ));
    }
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
    if input.render_profile != result.render_profile {
        return Err(format!(
            "OCR worker returned render profile `{}` for shard `{}` but input profile was `{}`",
            result.render_profile, result.shard_element_id, input.render_profile
        ));
    }
    if input.ocr_profile != result.ocr_profile {
        return Err(format!(
            "OCR worker returned OCR profile `{}` for shard `{}` but input profile was `{}`",
            result.ocr_profile, result.shard_element_id, input.ocr_profile
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/pdf_ocr_order.rs"]
mod tests;
