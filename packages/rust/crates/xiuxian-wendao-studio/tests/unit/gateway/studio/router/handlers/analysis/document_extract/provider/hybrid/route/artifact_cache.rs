use std::fs;

fn artifact_cache_lookup(root: &std::path::Path, planner: &str) -> impl Fn(&str) -> Option<String> {
    let root = root.to_path_buf();
    let planner = planner.to_string();
    move |key| match key {
        "WENDAO_DOCUMENT_EXTRACT_PDF_FULL_ARTIFACT_CACHE" => Some("enabled".to_string()),
        "WENDAO_ARTIFACT_CACHE_ROOT" => Some(root.to_string_lossy().to_string()),
        "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER" => Some(planner.clone()),
        _ => None,
    }
}

#[test]
fn full_artifact_cache_key_changes_when_profile_signature_changes() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("manual.pdf");
    fs::write(source.as_path(), b"%PDF fixture").map_err(|error| error.to_string())?;
    let root = temp.path().join("cache");
    let fast = artifact_cache_lookup(root.as_path(), "fast-risk-window");
    let ocr2 = artifact_cache_lookup(root.as_path(), "ocr2-risk-window");

    let fast_key = hybrid_page_ocr_artifact_cache_key_for_test(source.as_path(), &fast)?;
    let ocr2_key = hybrid_page_ocr_artifact_cache_key_for_test(source.as_path(), &ocr2)?;

    assert_ne!(fast_key, ocr2_key);
    Ok(())
}

#[test]
fn full_artifact_cache_roundtrips_and_rewrites_resource_paths() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("manual.pdf");
    fs::write(source.as_path(), b"%PDF fixture").map_err(|error| error.to_string())?;
    let first_output = temp.path().join("first-output");
    let second_output = temp.path().join("second-output");
    fs::create_dir_all(first_output.as_path()).map_err(|error| error.to_string())?;
    let markdown_path = first_output.join("manual.md");
    fs::write(markdown_path.as_path(), "# Manual\n").map_err(|error| error.to_string())?;
    let batch = sample_document_resource_batch(&[("document", 0, "# Manual\n", "document")])?;
    let mut columns = batch.columns().to_vec();
    columns[2] = Arc::new(StringArray::from(vec![
        markdown_path.to_string_lossy().to_string(),
    ])) as ArrayRef;
    let batch = RecordBatch::try_new(batch.schema(), columns).map_err(|error| error.to_string())?;
    write_arrow_file(
        first_output
            .join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME)
            .as_path(),
        &[batch],
    )?;
    fs::write(first_output.join("_complete.marker"), b"").map_err(|error| error.to_string())?;
    let cache_root = temp.path().join("artifact-cache");
    let lookup = artifact_cache_lookup(cache_root.as_path(), "fast-risk-window");

    assert!(store_hybrid_page_ocr_artifact_cache_for_test(
        source.as_path(),
        first_output.as_path(),
        &lookup
    )?);
    let response = hybrid_page_ocr_artifact_cache_response_for_test(
        source.as_path(),
        second_output.as_path(),
        &lookup,
    )?
    .ok_or_else(|| "expected full artifact cache hit".to_string())?;

    assert_eq!(response.batches.len(), 1);
    assert!(
        second_output
            .join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME)
            .exists()
    );
    let mirrored = read_arrow_file(
        second_output
            .join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME)
            .as_path(),
    )?;
    assert_eq!(
        test_string_value(&mirrored[0], "resourcePath", 0)?,
        second_output.join("manual.md").to_string_lossy()
    );
    Ok(())
}
