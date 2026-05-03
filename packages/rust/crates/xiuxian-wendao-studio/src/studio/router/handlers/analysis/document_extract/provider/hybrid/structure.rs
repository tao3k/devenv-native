use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};
use xiuxian_wendao_attachments::pdf::metrics::{
    DOCUMENT_METRICS_ARROW_CACHE_NAME, build_pdf_ocr_metrics_batch,
};
use xiuxian_wendao_attachments::pdf::structure::{
    DOCUMENT_STRUCTURE_ARROW_CACHE_NAME, DocumentStructureBlock, build_document_structure_batch,
    document_resource_batch_to_structure_blocks,
};

use super::precision_gate::validate_hybrid_precision_gate;
use super::types::HybridDocumentResourceBatch;
use crate::studio::router::handlers::analysis::document_extract::arrow_cache::{
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME, write_arrow_file,
};

pub(crate) fn write_hybrid_document_resource_artifacts(
    output: &Path,
    source: &Path,
    resource_batch: &HybridDocumentResourceBatch,
) -> Result<(), String> {
    let source_content_hash = sha256_file_hex(source)?;
    let structure_blocks = hybrid_document_structure_blocks(
        resource_batch,
        source_content_hash.as_str(),
        "wendao-hybrid-page-ocr",
    )?;
    validate_hybrid_precision_gate(
        resource_batch.page_count,
        resource_batch.text_page_indices.as_slice(),
        &resource_batch.batch,
        structure_blocks.as_slice(),
        resource_batch.ocr_inputs.as_slice(),
        resource_batch.ocr_results.as_slice(),
    )?;
    let structure_batch = build_document_structure_batch(structure_blocks.as_slice())?;
    let metrics_batch = build_pdf_ocr_metrics_batch(resource_batch.ocr_metrics.as_slice())?;
    write_arrow_file(
        output.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME).as_path(),
        std::slice::from_ref(&resource_batch.batch),
    )?;
    write_arrow_file(
        output.join(DOCUMENT_STRUCTURE_ARROW_CACHE_NAME).as_path(),
        std::slice::from_ref(&structure_batch),
    )?;
    write_arrow_file(
        output.join(DOCUMENT_METRICS_ARROW_CACHE_NAME).as_path(),
        std::slice::from_ref(&metrics_batch),
    )
}

pub(crate) fn hybrid_document_structure_blocks(
    resource_batch: &HybridDocumentResourceBatch,
    source_content_hash: &str,
    engine: &str,
) -> Result<Vec<DocumentStructureBlock>, String> {
    let mut blocks = document_resource_batch_to_structure_blocks(
        &resource_batch.batch,
        source_content_hash,
        engine,
    )?;
    if resource_batch.ocr_inputs.is_empty() {
        return Ok(blocks);
    }

    let inputs_by_shard = resource_batch
        .ocr_inputs
        .iter()
        .map(|input| (input.shard_element_id.as_str(), input))
        .collect::<HashMap<_, _>>();
    let results_by_element = resource_batch
        .ocr_results
        .iter()
        .map(|result| (result.element_id.as_str(), result))
        .collect::<HashMap<_, _>>();

    for block in &mut blocks {
        let Some(result) = results_by_element.get(block.resource_element_id.as_str()) else {
            continue;
        };
        let Some(input) = inputs_by_shard.get(result.shard_element_id.as_str()) else {
            continue;
        };
        block.reading_order_key = input.reading_order_key.clone();
        if let Some(block_index) = parse_reading_order_block_index(input.reading_order_key.as_str())
        {
            block.block_index = block_index;
        }
        block.block_type = match input.shard_type.as_str() {
            "region" => "ocr_region".to_string(),
            "page" => "ocr_page".to_string(),
            other => format!("ocr_{other}"),
        };
        block.parent_block_id = input.parent_shard_element_id.clone();
        block.confidence = result.confidence;
        block.bbox_left = Some(input.crop_left);
        block.bbox_top = Some(input.crop_top);
        block.bbox_right = Some(input.crop_right);
        block.bbox_bottom = Some(input.crop_bottom);
        block.provenance = serde_json::json!({
            "source": "pdf_ocr_shard",
            "shardType": input.shard_type,
            "regionIndex": input.region_index,
            "shardElementId": input.shard_element_id,
            "parentShardElementId": input.parent_shard_element_id,
            "readingOrderKey": input.reading_order_key,
            "rasterSha256": input.raster_sha256,
            "imagePath": input.image_path,
        })
        .to_string();
    }
    Ok(blocks)
}

fn parse_reading_order_block_index(reading_order_key: &str) -> Option<i32> {
    reading_order_key
        .split('.')
        .nth(1)
        .and_then(|value| value.parse::<i32>().ok())
}

fn sha256_file_hex(path: &Path) -> Result<String, String> {
    let mut file = File::open(path)
        .map_err(|error| format!("open `{}` for hashing: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("read `{}` for hashing: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
