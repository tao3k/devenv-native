use std::collections::{HashMap, HashSet};

use xiuxian_wendao_attachments::pdf::ocr::{
    PDF_OCR_BACKEND_TEXT_PROFILE, PdfOcrShardInput, PdfOcrShardResult, PdfOcrShardResultStatus,
};

const DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_EMPTY_PAGE_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_EMPTY_PAGE";
const BACKEND_TEXT_EMPTY_PAGE_VERIFIED_MODE: &str = "verified-empty";
const SOURCE_PDF_PAGE_IMAGE_MIME_TYPE: &str = "application/x-wendao-source-pdf-page";

#[cfg(test)]
pub(crate) fn validate_successful_ocr_results(
    results: &[PdfOcrShardResult],
    page_count: u32,
    shard_count: u32,
) -> Result<(), String> {
    validate_successful_ocr_results_inner(results, page_count, shard_count, None, "disabled")
}

pub(crate) fn validate_successful_ocr_results_for_inputs(
    results: &[PdfOcrShardResult],
    page_count: u32,
    shard_count: u32,
    inputs: &[PdfOcrShardInput],
) -> Result<(), String> {
    let mode = backend_text_empty_page_mode();
    validate_successful_ocr_results_inner(
        results,
        page_count,
        shard_count,
        Some(inputs),
        mode.as_str(),
    )
}

#[cfg(test)]
pub(crate) fn validate_successful_ocr_results_for_inputs_with_lookup(
    results: &[PdfOcrShardResult],
    page_count: u32,
    shard_count: u32,
    inputs: &[PdfOcrShardInput],
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<(), String> {
    let mode = backend_text_empty_page_mode_with_lookup(lookup);
    validate_successful_ocr_results_inner(
        results,
        page_count,
        shard_count,
        Some(inputs),
        mode.as_str(),
    )
}

fn validate_successful_ocr_results_inner(
    results: &[PdfOcrShardResult],
    page_count: u32,
    shard_count: u32,
    inputs: Option<&[PdfOcrShardInput]>,
    backend_text_empty_page_mode: &str,
) -> Result<(), String> {
    if results.len() != usize::try_from(shard_count).unwrap_or(usize::MAX) {
        return Err(format!(
            "OCR worker returned {} rows for {shard_count} rendered shards",
            results.len()
        ));
    }
    for (index, result) in results.iter().enumerate() {
        if result.page_index >= page_count {
            return Err(format!(
                "OCR worker returned out-of-range page {} for {page_count} page PDF",
                result.page_index
            ));
        }
        if result.status != PdfOcrShardResultStatus::Succeeded {
            let detail = result
                .error_message
                .as_deref()
                .map(str::trim)
                .filter(|message| !message.is_empty())
                .map_or_else(String::new, |message| format!(": {message}"));
            return Err(format!(
                "OCR worker returned non-success status `{}` for shard `{}` on page {}{}",
                result.status.as_str(),
                result.shard_element_id,
                result.page_index,
                detail
            ));
        }
        if result
            .text
            .as_deref()
            .is_none_or(|text| text.trim().is_empty())
            && !inputs
                .and_then(|inputs| inputs.get(index))
                .is_some_and(|input| {
                    allows_verified_empty_backend_text_page(input, backend_text_empty_page_mode)
                })
        {
            return Err(format!(
                "OCR worker returned empty text for shard `{}`",
                result.shard_element_id
            ));
        }
    }
    Ok(())
}

fn allows_verified_empty_backend_text_page(
    input: &PdfOcrShardInput,
    backend_text_empty_page_mode: &str,
) -> bool {
    backend_text_empty_page_mode == BACKEND_TEXT_EMPTY_PAGE_VERIFIED_MODE
        && input.ocr_profile == PDF_OCR_BACKEND_TEXT_PROFILE
        && input.shard_type == "page"
        && (input.image_mime_type == SOURCE_PDF_PAGE_IMAGE_MIME_TYPE
            || input.image_path.ends_with(".source-page-range"))
}

fn backend_text_empty_page_mode() -> String {
    backend_text_empty_page_mode_with_lookup(&|key| std::env::var(key).ok())
}

fn backend_text_empty_page_mode_with_lookup(lookup: &dyn Fn(&str) -> Option<String>) -> String {
    let value = lookup(DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_EMPTY_PAGE_ENV).unwrap_or_default();
    match value.trim().replace('_', "-").to_ascii_lowercase().as_str() {
        BACKEND_TEXT_EMPTY_PAGE_VERIFIED_MODE => BACKEND_TEXT_EMPTY_PAGE_VERIFIED_MODE.to_string(),
        _ => "disabled".to_string(),
    }
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
