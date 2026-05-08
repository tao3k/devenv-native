use std::path::Path;

use xiuxian_wendao_attachments::pdf::ocr::{
    PDF_OCR_BACKEND_TEXT_PROFILE, PDF_OCR_FAST_TEXT_PROFILE, PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
    PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PdfOcrShardInput, PdfOcrShardResult,
};
use xiuxian_wendao_attachments::pdf::render::{
    PdfPageBox, PdfPageRegionRenderRequest, PdfPageRenderProfile, PdfPageRenderShardReport,
};

use super::{
    DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY_ENV, HybridPdfFailedPageRecoveryMode,
    OCR2_REGION_SCAFFOLD_FILE_NAME, Ocr2RegionMaterializationStats, Ocr2RegionPipelineBatchKind,
    cached_ocr2_region_render_report, failed_page_recovery_candidates, failed_page_recovery_input,
    failed_page_recovery_mode_with_lookup, has_ocr2_recovery_page_candidates,
    materialize_hybrid_page_ocr_resource_batch_from_results, ocr2_region_render_cache_key,
    ocr2_region_scaffold_payload, record_ocr2_region_pipeline_batch_result,
    write_ocr2_region_scaffold_sidecar_with_lookup,
};
use crate::studio::router::handlers::analysis::document_extract::provider::hybrid::types::DOCUMENT_EXTRACT_PDF_HOSTED_VLM_SCAFFOLD_MODE_ENV;

#[test]
fn ocr2_region_scaffold_payload_is_disabled_by_default() -> Result<(), String> {
    let region = sample_region_input();

    let payload =
        ocr2_region_scaffold_payload(Path::new("/tmp/source.pdf"), &[region], false, &|_key| None);

    assert!(payload.is_none());
    Ok(())
}

#[test]
fn ocr2_region_scaffold_payload_records_region_fingerprints() -> Result<(), String> {
    let mut region = sample_region_input();
    region.parent_shard_element_id = "parent-page-shard".to_string();
    region.source_content_hash = "parent-page-hash".to_string();
    region.raster_sha256 = "region-raster-hash".to_string();
    region.render_dpi = 300;

    let payload = ocr2_region_scaffold_payload(
        Path::new("/tmp/source.pdf"),
        &[region],
        true,
        &scaffold_enabled_lookup,
    )
    .ok_or_else(|| "expected OCR2 scaffold payload".to_string())?;
    let items = payload
        .get("items")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "missing scaffold items".to_string())?;

    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["scaffoldKind"], "manual_region_candidate");
    assert_eq!(items[0]["parentShardElementId"], "parent-page-shard");
    assert_eq!(items[0]["sourceContentHash"], "parent-page-hash");
    assert_eq!(items[0]["rasterSha256"], "region-raster-hash");
    assert_eq!(items[0]["renderDpi"], 300);
    assert_eq!(items[0]["sourcePagePixelBox"]["right"], 1000);
    Ok(())
}

#[test]
fn ocr2_region_scaffold_sidecar_writes_only_when_enabled() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let region = sample_region_input();

    write_ocr2_region_scaffold_sidecar_with_lookup(
        Path::new("/tmp/source.pdf"),
        temp.path(),
        std::slice::from_ref(&region),
        false,
        &|_key| None,
    )?;
    assert!(!temp.path().join(OCR2_REGION_SCAFFOLD_FILE_NAME).exists());

    write_ocr2_region_scaffold_sidecar_with_lookup(
        Path::new("/tmp/source.pdf"),
        temp.path(),
        std::slice::from_ref(&region),
        false,
        &scaffold_enabled_lookup,
    )?;
    assert!(temp.path().join(OCR2_REGION_SCAFFOLD_FILE_NAME).is_file());
    Ok(())
}

#[test]
fn ocr2_region_candidate_detection_requires_direct_page_profile() {
    let mut input = sample_region_input();
    assert!(!has_ocr2_recovery_page_candidates(&[input.clone()]));

    input.shard_type = "page".to_string();
    input.ocr_profile = PDF_OCR_FAST_TEXT_PROFILE.to_string();
    assert!(!has_ocr2_recovery_page_candidates(&[input.clone()]));

    input.ocr_profile = PDF_OCR_HOSTED_VLM_DIRECT_PROFILE.to_string();
    assert!(has_ocr2_recovery_page_candidates(&[input]));
}

#[test]
fn failed_page_recovery_mode_accepts_only_hosted_vlm_page() {
    assert_eq!(
        failed_page_recovery_mode_with_lookup(&|_key| None),
        HybridPdfFailedPageRecoveryMode::Disabled,
    );
    assert_eq!(
        failed_page_recovery_mode_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY_ENV)
                .then(|| "hosted_vlm_page".to_string())
        }),
        HybridPdfFailedPageRecoveryMode::HostedVlmPage,
    );
    assert_eq!(
        failed_page_recovery_mode_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY_ENV)
                .then(|| "full-document".to_string())
        }),
        HybridPdfFailedPageRecoveryMode::Disabled,
    );
}

#[test]
fn failed_page_recovery_candidates_only_cover_failed_non_hosted_pages() {
    let mut failed_page = sample_region_input();
    failed_page.shard_type = "page".to_string();
    failed_page.ocr_profile = PDF_OCR_FAST_TEXT_PROFILE.to_string();
    failed_page.ocr_engine = "docling-fast-text-ocr".to_string();
    failed_page.shard_element_id = "failed-page".to_string();

    let mut empty_page = failed_page.clone();
    empty_page.page_index = 2;
    empty_page.shard_element_id = "empty-page".to_string();

    let mut hosted_failed_page = failed_page.clone();
    hosted_failed_page.page_index = 3;
    hosted_failed_page.ocr_profile = PDF_OCR_HOSTED_VLM_DIRECT_PROFILE.to_string();
    hosted_failed_page.ocr_engine = "hosted-vlm-direct-ocr".to_string();
    hosted_failed_page.shard_element_id = "hosted-failed-page".to_string();

    let mut failed_region = failed_page.clone();
    failed_region.page_index = 4;
    failed_region.shard_type = "region".to_string();
    failed_region.shard_element_id = "failed-region".to_string();

    let mut failed_backend_text_page = failed_page.clone();
    failed_backend_text_page.page_index = 5;
    failed_backend_text_page.ocr_profile = PDF_OCR_BACKEND_TEXT_PROFILE.to_string();
    failed_backend_text_page.ocr_engine = "docling-backend-text-ocr".to_string();
    failed_backend_text_page.shard_element_id = "failed-backend-text-page".to_string();

    let inputs = vec![
        failed_page.clone(),
        empty_page.clone(),
        hosted_failed_page.clone(),
        failed_region.clone(),
        failed_backend_text_page.clone(),
    ];
    let results = vec![
        PdfOcrShardResult::failed(&failed_page, "source page failed"),
        PdfOcrShardResult::succeeded(&empty_page, "   ", 1.0),
        PdfOcrShardResult::failed(&hosted_failed_page, "hosted page failed"),
        PdfOcrShardResult::failed(&failed_region, "region failed"),
        PdfOcrShardResult::failed(&failed_backend_text_page, "backend text failed"),
    ];

    let candidates = failed_page_recovery_candidates(inputs.as_slice(), results.as_slice());

    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].0, 0);
    assert_eq!(
        candidates[0].1.ocr_profile,
        PDF_OCR_HOSTED_VLM_DIRECT_PROFILE
    );
    assert_eq!(candidates[0].1.ocr_engine, "hosted-vlm-direct-ocr");
    assert_eq!(candidates[0].1.shard_element_id, "failed-page");
    assert_eq!(candidates[1].0, 1);
    assert_eq!(candidates[1].1.shard_element_id, "empty-page");
}

#[test]
fn failed_page_recovery_input_preserves_shard_identity() {
    let mut page = sample_region_input();
    page.shard_type = "page".to_string();
    page.ocr_profile = PDF_OCR_FAST_TEXT_PROFILE.to_string();
    page.ocr_engine = "docling-fast-text-ocr".to_string();

    let recovery = failed_page_recovery_input(&page);

    assert_eq!(recovery.shard_element_id, page.shard_element_id);
    assert_eq!(recovery.reading_order_key, page.reading_order_key);
    assert_eq!(recovery.ocr_profile, PDF_OCR_HOSTED_VLM_DIRECT_PROFILE);
    assert_eq!(recovery.ocr_engine, "hosted-vlm-direct-ocr");
}

#[test]
fn ocr2_region_render_cache_key_tracks_source_profile_and_region() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("source.pdf");
    std::fs::write(source.as_path(), b"source-a").map_err(|error| error.to_string())?;
    let profile = PdfPageRenderProfile::ocr_default();
    let region = sample_region_request(1);

    let baseline =
        ocr2_region_render_cache_key(source.as_path(), &profile, std::slice::from_ref(&region))?;
    assert_eq!(
        baseline,
        ocr2_region_render_cache_key(source.as_path(), &profile, std::slice::from_ref(&region),)?
    );

    let mut changed_region = region.clone();
    changed_region.region_box = PdfPageBox::new(10.0, 10.0, 220.0, 260.0);
    assert_ne!(
        baseline,
        ocr2_region_render_cache_key(source.as_path(), &profile, &[changed_region])?
    );

    let mut changed_profile = profile.clone();
    changed_profile.dpi = 360;
    assert_ne!(
        baseline,
        ocr2_region_render_cache_key(
            source.as_path(),
            &changed_profile,
            std::slice::from_ref(&region),
        )?
    );

    std::fs::write(source.as_path(), b"source-b").map_err(|error| error.to_string())?;
    assert_ne!(
        baseline,
        ocr2_region_render_cache_key(source.as_path(), &profile, &[region])?
    );
    Ok(())
}

#[test]
fn cached_ocr2_region_render_report_rejects_missing_artifacts() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("source.pdf");
    std::fs::write(source.as_path(), b"source").map_err(|error| error.to_string())?;

    let cached = cached_ocr2_region_render_report(
        source.as_path(),
        temp.path().join("missing").as_path(),
        1,
        &PdfPageRenderProfile::ocr_default(),
        1,
    );

    assert!(cached.is_none());
    Ok(())
}

#[test]
fn hybrid_page_ocr_resource_batch_orders_split_pipeline_results() -> Result<(), String> {
    let mut page = sample_region_input();
    page.page_index = 0;
    page.shard_type = "page".to_string();
    page.shard_element_id = "page-shard".to_string();
    page.parent_shard_element_id.clear();
    page.ocr_profile = PDF_OCR_FAST_TEXT_PROFILE.to_string();
    page.ocr_engine = "docling-fast-text-ocr".to_string();

    let mut region = sample_region_input();
    region.page_index = 0;
    region.shard_element_id = "region-shard".to_string();
    region.parent_shard_element_id = page.shard_element_id.clone();
    let inputs = vec![page.clone(), region.clone()];
    let results = vec![
        PdfOcrShardResult::succeeded(&region, "region text", 1.0),
        PdfOcrShardResult::succeeded(&page, "page text", 1.0),
    ];

    let batch = materialize_hybrid_page_ocr_resource_batch_from_results(
        &sample_render_report(),
        inputs,
        results,
        42.0,
    )?;

    assert_eq!(batch.ocr_results[0].shard_element_id, "page-shard");
    assert_eq!(batch.ocr_results[1].shard_element_id, "region-shard");
    assert_eq!(batch.ocr_metrics.len(), 2);
    assert_eq!(batch.page_count, 1);
    Ok(())
}

#[test]
fn ocr2_region_pipeline_batch_result_telemetry_splits_base_and_region() {
    let mut phases = std::collections::BTreeMap::new();
    let mut stats = Ocr2RegionMaterializationStats::default();

    record_ocr2_region_pipeline_batch_result(
        &mut phases,
        &mut stats,
        Ocr2RegionPipelineBatchKind::Base,
        21,
        1_250.0,
    );
    record_ocr2_region_pipeline_batch_result(
        &mut phases,
        &mut stats,
        Ocr2RegionPipelineBatchKind::Region,
        3,
        2_500.0,
    );
    record_ocr2_region_pipeline_batch_result(
        &mut phases,
        &mut stats,
        Ocr2RegionPipelineBatchKind::Region,
        2,
        3_000.0,
    );

    assert_eq!(stats.pipeline_base_result_count, 1);
    assert_eq!(stats.pipeline_base_result_shard_count, 21);
    assert_eq!(stats.pipeline_region_result_count, 2);
    assert_eq!(stats.pipeline_region_result_shard_count, 5);
    assert_eq!(phases["regionPipelineFirstBaseResult"], 1_250.0);
    assert_eq!(phases["regionPipelineLastBaseResult"], 1_250.0);
    assert_eq!(phases["regionPipelineFirstRegionResult"], 2_500.0);
    assert_eq!(phases["regionPipelineLastRegionResult"], 3_000.0);
}

fn scaffold_enabled_lookup(key: &str) -> Option<String> {
    (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_SCAFFOLD_MODE_ENV)
        .then(|| "region-table-json".to_string())
}

fn sample_region_request(region_index: u32) -> PdfPageRegionRenderRequest {
    sample_region_request_for_page(1, region_index)
}

fn sample_region_request_for_page(
    page_index: u32,
    region_index: u32,
) -> PdfPageRegionRenderRequest {
    PdfPageRegionRenderRequest::new(
        page_index,
        region_index,
        PdfPageBox::new(10.0, 20.0, 110.0, 220.0),
        Some(format!("{page_index:06}.{region_index:06}")),
    )
}

fn sample_render_report() -> PdfPageRenderShardReport {
    PdfPageRenderShardReport {
        source_path: "/tmp/source.pdf".to_string(),
        output_dir: "/tmp/out".to_string(),
        page_count: 1,
        shard_count: 2,
        manifest_arrow_path: None,
        ocr_input_arrow_path: None,
        pending_resource_arrow_path: None,
        render_profile: "pdfium-render-page-shards-v1".to_string(),
        render_selection: "region_shards".to_string(),
        status: "rendered".to_string(),
        routing_decision: "hybrid_page_ocr_candidate".to_string(),
        elapsed_ms: 0.0,
        error_message: None,
    }
}

fn sample_region_input() -> PdfOcrShardInput {
    PdfOcrShardInput {
        contract_version: PDF_OCR_SHARD_INPUT_SCHEMA_VERSION.to_string(),
        source_path: "/tmp/source.pdf".to_string(),
        source_content_hash: "sourcehash".to_string(),
        page_index: 1,
        image_path: "/tmp/out/_ocr2-region-renders/page-00001-region-00001.png".to_string(),
        image_mime_type: "image/png".to_string(),
        raster_sha256: "raster-1".to_string(),
        render_profile: "pdfium-render-page-shards-v1".to_string(),
        ocr_profile: PDF_OCR_HOSTED_VLM_DIRECT_PROFILE.to_string(),
        ocr_engine: "hosted-vlm-direct-ocr".to_string(),
        preferred_languages: vec!["auto".to_string()],
        min_confidence: 0.0,
        preserve_layout: true,
        raster_width_px: 1000,
        raster_height_px: 1000,
        render_dpi: 300,
        rotation_degrees: 0,
        crop_left: 10.0,
        crop_bottom: 20.0,
        crop_right: 110.0,
        crop_top: 220.0,
        point_to_pixel_scale_x: 3.0,
        point_to_pixel_scale_y: 3.0,
        shard_element_id: "region-shard".to_string(),
        shard_type: "region".to_string(),
        region_index: 1,
        parent_shard_element_id: "parent-shard".to_string(),
        reading_order_key: "000001.000050".to_string(),
        source_page_pixel_left: 0,
        source_page_pixel_top: 100,
        source_page_pixel_right: 1000,
        source_page_pixel_bottom: 900,
    }
}
