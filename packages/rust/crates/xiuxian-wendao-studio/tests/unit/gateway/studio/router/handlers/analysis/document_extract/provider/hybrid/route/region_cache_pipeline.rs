#[test]
fn direct_docling_structure_recovery_requires_no_region_controls() {
    assert!(
        direct_docling_structure_recovery_page_range_enabled_with_lookup(&direct_docling_lookup)
    );
    assert!(
        !direct_docling_structure_recovery_page_range_enabled_with_lookup(&|key| {
            if key == "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER" {
                Some("profile-risk-window".to_string())
            } else {
                direct_docling_lookup(key)
            }
        },)
    );
    assert!(
        !direct_docling_structure_recovery_page_range_enabled_with_lookup(&|key| {
            if key == "WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_JSON" {
                Some("[]".to_string())
            } else {
                direct_docling_lookup(key)
            }
        },)
    );
    assert!(
        !direct_docling_structure_recovery_page_range_enabled_with_lookup(&|key| {
            if key == "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PIPELINE" {
                Some("render-dispatch".to_string())
            } else {
                direct_docling_lookup(key)
            }
        },)
    );
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

