use super::{
    Duration, PdfOcrShardCache, PdfOcrShardCachePolicy, PdfOcrShardInput, PdfOcrShardResult,
    ocr_shard_cache_key,
};
use xiuxian_wendao_attachments::pdf::ocr::PDF_OCR_SHARD_INPUT_SCHEMA_VERSION;

#[test]
fn cache_key_changes_for_page_region_profile_and_raster() {
    let page = sample_ocr_input(0, "page");
    let mut other_page = sample_ocr_input(1, "page");
    let mut region = sample_ocr_input(0, "region");
    let mut profile = sample_ocr_input(0, "page");
    let mut raster = sample_ocr_input(0, "page");
    other_page.shard_element_id = "page-shard-1".to_string();
    region.region_index = 3;
    region.shard_element_id = "region-shard-0-3".to_string();
    profile.ocr_profile = "docling-fast-text-ocr".to_string();
    raster.raster_sha256 = "different-raster".to_string();

    let base = ocr_shard_cache_key(&page);

    assert_ne!(base, ocr_shard_cache_key(&other_page));
    assert_ne!(base, ocr_shard_cache_key(&region));
    assert_ne!(base, ocr_shard_cache_key(&profile));
    assert_ne!(base, ocr_shard_cache_key(&raster));
}

#[test]
fn cache_roundtrips_successful_result() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let cache = PdfOcrShardCache::new(temp.path().to_path_buf());
    let input = sample_ocr_input(0, "page");
    let result = PdfOcrShardResult::succeeded(&input, "cached text", 0.97);

    assert!(cache.store_successful(&input, &result)?);
    let resolution = cache.resolve(std::slice::from_ref(&input));
    let merged = resolution.merge(Vec::new())?;

    assert_eq!(merged, vec![result]);
    Ok(())
}

#[test]
fn cache_merges_hits_and_live_misses_in_input_order() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let cache = PdfOcrShardCache::new(temp.path().to_path_buf());
    let inputs = vec![
        sample_ocr_input(0, "page"),
        sample_ocr_input(1, "page"),
        sample_ocr_input(2, "page"),
    ];
    let hit_zero = PdfOcrShardResult::succeeded(&inputs[0], "cached 0", 1.0);
    let hit_two = PdfOcrShardResult::succeeded(&inputs[2], "cached 2", 1.0);
    cache.store_successful(&inputs[0], &hit_zero)?;
    cache.store_successful(&inputs[2], &hit_two)?;

    let resolution = cache.resolve(inputs.as_slice());

    assert_eq!(resolution.hit_count(), 2);
    assert_eq!(resolution.misses().len(), 1);
    assert_eq!(resolution.misses()[0].shard_element_id, "page-shard-1");

    let live = vec![PdfOcrShardResult::succeeded(&inputs[1], "live 1", 1.0)];
    let merged = resolution.merge(live)?;

    assert_eq!(merged[0].text.as_deref(), Some("cached 0"));
    assert_eq!(merged[1].text.as_deref(), Some("live 1"));
    assert_eq!(merged[2].text.as_deref(), Some("cached 2"));
    Ok(())
}

#[test]
fn cache_does_not_persist_failed_results() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let cache = PdfOcrShardCache::new(temp.path().to_path_buf());
    let input = sample_ocr_input(0, "page");
    let failed = PdfOcrShardResult::failed(&input, "transient failure");

    assert!(!cache.store_successful(&input, &failed)?);
    let resolution = cache.resolve(std::slice::from_ref(&input));

    assert_eq!(resolution.hit_count(), 0);
    assert_eq!(resolution.misses().len(), 1);
    Ok(())
}

#[test]
fn cache_prunes_oldest_entries_when_entry_limit_is_exceeded() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let cache = PdfOcrShardCache::new_with_policy(
        temp.path().to_path_buf(),
        PdfOcrShardCachePolicy {
            max_bytes: None,
            max_entries: Some(2),
            max_age: None,
            sweep_interval: Duration::ZERO,
        },
    );
    let inputs = vec![
        sample_ocr_input(0, "page"),
        sample_ocr_input(1, "page"),
        sample_ocr_input(2, "page"),
    ];
    for input in &inputs {
        cache.store_successful(
            input,
            &PdfOcrShardResult::succeeded(input, format!("page {}", input.page_index), 1.0),
        )?;
        std::thread::sleep(Duration::from_millis(2));
    }

    let report = cache.prune()?;
    let resolution = cache.resolve(inputs.as_slice());

    assert!(report.retained_entries <= 2);
    assert_eq!(resolution.hit_count(), 2);
    assert_eq!(resolution.misses().len(), 1);
    assert_eq!(resolution.misses()[0].shard_element_id, "page-shard-0");
    Ok(())
}

#[test]
fn cache_prunes_oldest_entries_when_byte_limit_is_exceeded() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let cache = PdfOcrShardCache::new_with_policy(
        temp.path().to_path_buf(),
        PdfOcrShardCachePolicy {
            max_bytes: Some(1),
            max_entries: None,
            max_age: None,
            sweep_interval: Duration::ZERO,
        },
    );
    let input = sample_ocr_input(0, "page");

    cache.store_successful(&input, &PdfOcrShardResult::succeeded(&input, "page", 1.0))?;
    let report = cache.prune()?;
    let resolution = cache.resolve(std::slice::from_ref(&input));

    assert_eq!(report.retained_entries, 0);
    assert_eq!(resolution.hit_count(), 0);
    assert_eq!(resolution.misses().len(), 1);
    Ok(())
}

#[test]
fn cache_prunes_expired_entries() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let cache = PdfOcrShardCache::new_with_policy(
        temp.path().to_path_buf(),
        PdfOcrShardCachePolicy {
            max_bytes: None,
            max_entries: None,
            max_age: Some(Duration::ZERO),
            sweep_interval: Duration::ZERO,
        },
    );
    let input = sample_ocr_input(0, "page");

    cache.store_successful(&input, &PdfOcrShardResult::succeeded(&input, "page", 1.0))?;
    std::thread::sleep(Duration::from_millis(2));
    let report = cache.prune()?;
    let resolution = cache.resolve(std::slice::from_ref(&input));

    assert_eq!(report.retained_entries, 0);
    assert_eq!(resolution.hit_count(), 0);
    Ok(())
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
