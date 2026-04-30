use std::collections::{HashMap, HashSet};

use xiuxian_wendao_attachments::pdf::ocr::{
    PdfOcrShardInput, PdfOcrShardResult, PdfOcrShardResultStatus,
};

pub(in super::super) fn validate_successful_ocr_results(
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
    }
    Ok(())
}

pub(in super::super) fn validate_ocr_results_match_inputs(
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

pub(in super::super) fn validate_hybrid_page_coverage(
    page_count: u32,
    text_page_indices: &[u32],
    ocr_results: &[PdfOcrShardResult],
) -> Result<(), String> {
    if let Some(page_index) = text_page_indices
        .iter()
        .copied()
        .find(|page_index| *page_index >= page_count)
    {
        return Err(format!(
            "native text page {page_index} is out of range for {page_count} page PDF"
        ));
    }
    let mut covered = text_page_indices.iter().copied().collect::<HashSet<_>>();
    for result in ocr_results {
        if covered.contains(&result.page_index) {
            return Err(format!(
                "hybrid merge has duplicate page coverage for page {}",
                result.page_index
            ));
        }
        covered.insert(result.page_index);
    }
    let missing = (0..page_count)
        .filter(|page_index| !covered.contains(page_index))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "hybrid merge is missing page coverage: {missing:?}"
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(in super::super) fn validate_hybrid_shard_coverage(
    page_count: u32,
    text_page_indices: &[u32],
    ocr_inputs: &[PdfOcrShardInput],
    ocr_results: &[PdfOcrShardResult],
) -> Result<(), String> {
    validate_ocr_results_match_inputs(ocr_inputs, ocr_results)?;
    if let Some(page_index) = text_page_indices
        .iter()
        .copied()
        .find(|page_index| *page_index >= page_count)
    {
        return Err(format!(
            "native text page {page_index} is out of range for {page_count} page PDF"
        ));
    }

    let mut covered_pages = HashSet::new();
    for page_index in text_page_indices {
        if !covered_pages.insert(*page_index) {
            return Err(format!(
                "hybrid merge has duplicate native text page coverage for page {page_index}"
            ));
        }
    }

    for input in ocr_inputs {
        match input.shard_type.as_str() {
            "page" => {
                if !covered_pages.insert(input.page_index) {
                    return Err(format!(
                        "hybrid merge has duplicate page coverage for page {}",
                        input.page_index
                    ));
                }
            }
            "region" => {
                if !covered_pages.contains(&input.page_index) {
                    return Err(format!(
                        "region OCR shard `{}` has no native text coverage for page {}",
                        input.shard_element_id, input.page_index
                    ));
                }
            }
            other => {
                return Err(format!(
                    "unsupported OCR shard input type `{other}` for shard `{}`",
                    input.shard_element_id
                ));
            }
        }
    }

    let missing = (0..page_count)
        .filter(|page_index| !covered_pages.contains(page_index))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "hybrid merge is missing page coverage: {missing:?}"
        ));
    }
    Ok(())
}
