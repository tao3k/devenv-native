use std::collections::{HashMap, HashSet};

use xiuxian_wendao_attachments::pdf::ocr::{PdfOcrShardInput, PdfOcrShardResult};
use xiuxian_wendao_attachments::pdf::structure::DocumentStructureBlock;

const OCR_REGION_PATCH_PROTOCOL: &str = "sentinel-sidecar-v1";

pub(super) fn validate_structure_rows(
    page_count: u32,
    structure_blocks: &[DocumentStructureBlock],
    ocr_inputs: &[PdfOcrShardInput],
    ocr_results: &[PdfOcrShardResult],
) -> Result<(), String> {
    if structure_blocks.is_empty() {
        return Err("hybrid precision gate rejected an empty structure sidecar".to_string());
    }
    validate_structure_block_identity(page_count, structure_blocks)?;
    validate_ocr_structure_provenance(structure_blocks, ocr_inputs, ocr_results)
}

fn validate_structure_block_identity(
    page_count: u32,
    structure_blocks: &[DocumentStructureBlock],
) -> Result<(), String> {
    let mut block_ids = HashSet::new();
    for block in structure_blocks {
        if block.block_id.trim().is_empty() {
            return Err("hybrid structure block has an empty block id".to_string());
        }
        if !block_ids.insert(block.block_id.as_str()) {
            return Err(format!(
                "hybrid structure has duplicate block id `{}`",
                block.block_id
            ));
        }
        let page_index = u32::try_from(block.page_index).map_err(|_| {
            format!(
                "hybrid structure block `{}` has negative page index {}",
                block.block_id, block.page_index
            )
        })?;
        if page_index >= page_count {
            return Err(format!(
                "hybrid structure block `{}` has out-of-range page {} for {page_count} page PDF",
                block.block_id, block.page_index
            ));
        }
        if block.reading_order_key.trim().is_empty() {
            return Err(format!(
                "hybrid structure block `{}` has an empty reading order key",
                block.block_id
            ));
        }
    }
    Ok(())
}

fn validate_ocr_structure_provenance(
    structure_blocks: &[DocumentStructureBlock],
    ocr_inputs: &[PdfOcrShardInput],
    ocr_results: &[PdfOcrShardResult],
) -> Result<(), String> {
    let inputs_by_shard = ocr_inputs
        .iter()
        .map(|input| (input.shard_element_id.as_str(), input))
        .collect::<HashMap<_, _>>();
    let blocks_by_element = structure_blocks
        .iter()
        .map(|block| (block.resource_element_id.as_str(), block))
        .collect::<HashMap<_, _>>();

    for result in ocr_results {
        let block = blocks_by_element
            .get(result.element_id.as_str())
            .ok_or_else(|| {
                format!(
                    "hybrid structure is missing OCR result block `{}`",
                    result.element_id
                )
            })?;
        if block.bbox_left.is_none()
            || block.bbox_top.is_none()
            || block.bbox_right.is_none()
            || block.bbox_bottom.is_none()
        {
            return Err(format!(
                "hybrid structure OCR block `{}` is missing bbox provenance",
                block.block_id
            ));
        }
        let input = inputs_by_shard
            .get(result.shard_element_id.as_str())
            .ok_or_else(|| {
                format!(
                    "hybrid structure cannot resolve OCR shard `{}`",
                    result.shard_element_id
                )
            })?;
        if !block.provenance.contains(input.shard_element_id.as_str()) {
            return Err(format!(
                "hybrid structure OCR block `{}` does not preserve shard provenance",
                block.block_id
            ));
        }
        if block.block_type == "ocr_region" {
            validate_region_patch_provenance(block, input)?;
        }
    }
    Ok(())
}

fn validate_region_patch_provenance(
    block: &DocumentStructureBlock,
    input: &PdfOcrShardInput,
) -> Result<(), String> {
    if block.parent_block_id.trim().is_empty() {
        return Err(format!(
            "hybrid structure OCR region block `{}` has no parent shard id",
            block.block_id
        ));
    }
    let provenance =
        serde_json::from_str::<serde_json::Value>(block.provenance.as_str()).map_err(|error| {
            format!(
                "hybrid structure OCR region block `{}` has invalid provenance JSON: {error}",
                block.block_id
            )
        })?;
    if provenance
        .get("parentShardElementId")
        .and_then(serde_json::Value::as_str)
        != Some(block.parent_block_id.as_str())
    {
        return Err(format!(
            "hybrid structure OCR region block `{}` does not preserve parent shard provenance",
            block.block_id
        ));
    }
    if provenance
        .get("patchProtocol")
        .and_then(serde_json::Value::as_str)
        != Some(OCR_REGION_PATCH_PROTOCOL)
    {
        return Err(format!(
            "hybrid structure OCR region block `{}` is missing sentinel patch protocol",
            block.block_id
        ));
    }
    if block.parent_block_id != input.parent_shard_element_id {
        return Err(format!(
            "hybrid structure OCR region block `{}` parent shard does not match OCR input",
            block.block_id
        ));
    }
    Ok(())
}
