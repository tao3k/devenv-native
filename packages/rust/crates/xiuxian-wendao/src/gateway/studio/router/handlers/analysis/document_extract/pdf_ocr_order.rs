use std::collections::{HashMap, HashSet};

use xiuxian_wendao_attachments::pdf::ocr::{PdfOcrShardInput, PdfOcrShardResult};

pub(super) fn order_ocr_results_by_inputs(
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

    let mut input_shards = HashSet::new();
    for input in inputs {
        if !input_shards.insert(input.shard_element_id.as_str()) {
            return Err(format!(
                "duplicate OCR shard input id `{}`",
                input.shard_element_id
            ));
        }
    }

    let mut results_by_shard = HashMap::with_capacity(results.len());
    for result in results {
        let shard_id = result.shard_element_id.clone();
        if results_by_shard.insert(shard_id.clone(), result).is_some() {
            return Err(format!("duplicate OCR shard result id `{shard_id}`"));
        }
    }

    let mut ordered = Vec::with_capacity(inputs.len());
    for input in inputs {
        let result = results_by_shard
            .remove(input.shard_element_id.as_str())
            .ok_or_else(|| {
                format!(
                    "OCR worker did not return shard id `{}`",
                    input.shard_element_id
                )
            })?;
        validate_ocr_result_matches_input(input, &result)?;
        ordered.push(result);
    }

    if let Some(unknown) = results_by_shard.keys().next() {
        return Err(format!("OCR worker returned unknown shard id `{unknown}`"));
    }

    Ok(ordered)
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
mod tests {
    use super::*;
    use xiuxian_wendao_attachments::pdf::ocr::{
        PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PdfOcrShardResult,
    };

    #[test]
    fn restores_ocr_results_to_input_order() -> Result<(), String> {
        let inputs = vec![
            sample_ocr_input(0, "page"),
            sample_ocr_input(1, "page"),
            sample_ocr_input(2, "page"),
        ];
        let results = vec![
            PdfOcrShardResult::succeeded(&inputs[2], "page 2", 1.0),
            PdfOcrShardResult::succeeded(&inputs[0], "page 0", 1.0),
            PdfOcrShardResult::succeeded(&inputs[1], "page 1", 1.0),
        ];

        let ordered = order_ocr_results_by_inputs(inputs.as_slice(), results)?;

        assert_eq!(ordered[0].shard_element_id, "page-shard-0");
        assert_eq!(ordered[1].shard_element_id, "page-shard-1");
        assert_eq!(ordered[2].shard_element_id, "page-shard-2");
        Ok(())
    }

    #[test]
    fn rejects_duplicate_ocr_result_shards() {
        let inputs = vec![sample_ocr_input(0, "page"), sample_ocr_input(1, "page")];
        let duplicate = PdfOcrShardResult::succeeded(&inputs[0], "page", 1.0);
        let error = match order_ocr_results_by_inputs(
            inputs.as_slice(),
            vec![duplicate.clone(), duplicate],
        ) {
            Ok(_) => panic!("expected duplicate result to fail"),
            Err(error) => error,
        };

        assert!(error.contains("duplicate OCR shard result id"));
    }

    #[test]
    fn rejects_mismatched_ocr_result_hashes() {
        let input = sample_ocr_input(0, "page");
        let mut result = PdfOcrShardResult::succeeded(&input, "page", 1.0);
        result.raster_sha256 = "different".to_string();

        let error = match validate_ocr_result_matches_input(&input, &result) {
            Ok(()) => panic!("expected mismatched raster hash to fail"),
            Err(error) => error,
        };

        assert!(error.contains("raster hash"));
    }

    fn sample_ocr_input(page_index: u32, shard_type: &str) -> PdfOcrShardInput {
        PdfOcrShardInput {
            contract_version: PDF_OCR_SHARD_INPUT_SCHEMA_VERSION.to_string(),
            source_path: "/tmp/source.pdf".to_string(),
            source_content_hash: "sourcehash".to_string(),
            page_index,
            image_path: format!("/tmp/page-{page_index:05}.png"),
            image_mime_type: "image/png".to_string(),
            raster_sha256: format!("rasterhash-{page_index}"),
            render_profile: "pdfium-render-page-shards-v1".to_string(),
            ocr_profile: "docling-compatible-page-ocr-v1".to_string(),
            ocr_engine: "docling-compatible-ocr".to_string(),
            preferred_languages: vec!["auto".to_string()],
            min_confidence: 0.0,
            preserve_layout: true,
            raster_width_px: 2400,
            raster_height_px: 3100,
            render_dpi: 300,
            rotation_degrees: 0,
            crop_left: 0.0,
            crop_bottom: 0.0,
            crop_right: 612.0,
            crop_top: 792.0,
            point_to_pixel_scale_x: 3.921_568_627,
            point_to_pixel_scale_y: 3.914_141_414,
            shard_element_id: format!("{shard_type}-shard-{page_index}"),
            shard_type: shard_type.to_string(),
            region_index: 0,
            parent_shard_element_id: String::new(),
            reading_order_key: format!("{page_index:06}.000000"),
            source_page_pixel_left: 0,
            source_page_pixel_top: 0,
            source_page_pixel_right: 2400,
            source_page_pixel_bottom: 3100,
        }
    }
}
