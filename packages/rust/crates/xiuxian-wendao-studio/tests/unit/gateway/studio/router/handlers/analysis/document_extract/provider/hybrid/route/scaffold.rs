#[test]
fn ocr2_region_scaffold_payload_is_disabled_by_default() {
    let region = sample_region_input();

    let payload =
        ocr2_region_scaffold_payload(Path::new("/tmp/source.pdf"), &[region], false, &|_key| None);

    assert!(payload.is_none());
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
