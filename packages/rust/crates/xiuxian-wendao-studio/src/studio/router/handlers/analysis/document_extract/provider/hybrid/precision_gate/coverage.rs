use std::collections::{HashMap, HashSet};

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
    let mut coverage = HybridShardCoverage::from_native_text_pages(page_count, text_page_indices)?;
    coverage.add_page_shards(ocr_inputs)?;
    coverage.validate_region_shards(ocr_inputs)?;
    coverage.validate_complete(page_count)?;
    Ok(())
}

struct HybridShardCoverage<'a> {
    covered_pages: HashSet<u32>,
    page_shards_by_id: HashMap<&'a str, u32>,
    page_shard_ids_by_page: HashMap<u32, HashSet<&'a str>>,
}

impl<'a> HybridShardCoverage<'a> {
    fn from_native_text_pages(page_count: u32, text_page_indices: &[u32]) -> Result<Self, String> {
        let mut coverage = Self {
            covered_pages: HashSet::new(),
            page_shards_by_id: HashMap::new(),
            page_shard_ids_by_page: HashMap::new(),
        };
        for page_index in text_page_indices {
            coverage.add_native_text_page(page_count, *page_index)?;
        }
        Ok(coverage)
    }

    fn add_native_text_page(&mut self, page_count: u32, page_index: u32) -> Result<(), String> {
        if page_index >= page_count {
            return Err(format!(
                "native text page {page_index} is out of range for {page_count} page PDF"
            ));
        }
        if !self.covered_pages.insert(page_index) {
            return Err(format!(
                "hybrid merge has duplicate native text page coverage for page {page_index}"
            ));
        }
        Ok(())
    }

    fn add_page_shards(&mut self, ocr_inputs: &'a [PdfOcrShardInput]) -> Result<(), String> {
        for input in ocr_inputs.iter().filter(|input| input.shard_type == "page") {
            self.add_page_shard(input)?;
        }
        Ok(())
    }

    fn add_page_shard(&mut self, input: &'a PdfOcrShardInput) -> Result<(), String> {
        if input.shard_element_id.trim().is_empty() {
            return Err(format!(
                "page OCR shard for page {} has an empty shard id",
                input.page_index
            ));
        }
        if self
            .page_shards_by_id
            .insert(input.shard_element_id.as_str(), input.page_index)
            .is_some()
        {
            return Err(format!(
                "hybrid merge has duplicate page shard id `{}`",
                input.shard_element_id
            ));
        }
        self.page_shard_ids_by_page
            .entry(input.page_index)
            .or_default()
            .insert(input.shard_element_id.as_str());
        if !self.covered_pages.insert(input.page_index) {
            return Err(format!(
                "hybrid merge has duplicate page coverage for page {}",
                input.page_index
            ));
        }
        Ok(())
    }

    fn validate_region_shards(&self, ocr_inputs: &[PdfOcrShardInput]) -> Result<(), String> {
        for input in ocr_inputs {
            match input.shard_type.as_str() {
                "page" => {}
                "region" => validate_region_parent_binding(
                    input,
                    &self.covered_pages,
                    &self.page_shards_by_id,
                    &self.page_shard_ids_by_page,
                )?,
                other => {
                    return Err(format!(
                        "unsupported OCR shard input type `{other}` for shard `{}`",
                        input.shard_element_id
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_complete(&self, page_count: u32) -> Result<(), String> {
        let missing = (0..page_count)
            .filter(|page_index| !self.covered_pages.contains(page_index))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(format!(
                "hybrid merge is missing page coverage: {missing:?}"
            ));
        }
        Ok(())
    }
}

fn validate_region_parent_binding(
    input: &PdfOcrShardInput,
    covered_pages: &HashSet<u32>,
    page_shards_by_id: &HashMap<&str, u32>,
    page_shard_ids_by_page: &HashMap<u32, HashSet<&str>>,
) -> Result<(), String> {
    if !covered_pages.contains(&input.page_index) {
        return Err(format!(
            "region OCR shard `{}` has no native text coverage for page {}",
            input.shard_element_id, input.page_index
        ));
    }
    if input.parent_shard_element_id.trim().is_empty() {
        return Err(format!(
            "region OCR shard `{}` has no parent page shard id",
            input.shard_element_id
        ));
    }
    if let Some(parent_page_index) = page_shards_by_id.get(input.parent_shard_element_id.as_str())
        && *parent_page_index != input.page_index
    {
        return Err(format!(
            "region OCR shard `{}` points to parent page shard `{}` on page {}, not page {}",
            input.shard_element_id,
            input.parent_shard_element_id,
            parent_page_index,
            input.page_index
        ));
    }
    if !page_shards_by_id.contains_key(input.parent_shard_element_id.as_str())
        && page_shard_ids_by_page.contains_key(&input.page_index)
    {
        return Err(format!(
            "region OCR shard `{}` parent page shard `{}` does not match page {} coverage",
            input.shard_element_id, input.parent_shard_element_id, input.page_index
        ));
    }
    Ok(())
}
