use std::collections::HashSet;

use xiuxian_wendao_attachments::pdf::ocr::{PdfOcrShardInput, PdfOcrShardResult};

use super::ocr::validate_ocr_results_match_inputs;

pub(crate) fn validate_hybrid_page_coverage(
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

pub(crate) fn validate_hybrid_shard_coverage(
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
