use super::{
    Arc, DocumentExtractJobRegistry, StudioDocumentExtractFlightRouteProvider,
    document_extract_batches_are_cacheable, fs, read_arrow_file, test_document_resource_batch,
    write_arrow_file,
};

#[tokio::test]
async fn sync_document_extract_reuses_succeeded_content_artifact() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("scan.png");
    fs::write(source.as_path(), b"image fixture").map_err(|error| error.to_string())?;
    let registry = DocumentExtractJobRegistry::new(
        temp.path().join("jobs.duckdb"),
        temp.path().join("artifacts"),
    )?;
    let provider = StudioDocumentExtractFlightRouteProvider::from_registry(Ok(registry), 1);
    let first_output = temp.path().join("first-output");
    fs::create_dir_all(first_output.as_path()).map_err(|error| error.to_string())?;
    let first_markdown = first_output.join("scan.md");
    fs::write(first_markdown.as_path(), "# Scan\n").map_err(|error| error.to_string())?;
    let batch = test_document_resource_batch(
        source.to_string_lossy().as_ref(),
        first_markdown.to_string_lossy().as_ref(),
    )?;
    write_arrow_file(
        first_output.join("_resources.arrow").as_path(),
        std::slice::from_ref(&batch),
    )?;
    fs::write(first_output.join("_complete.marker"), b"").map_err(|error| error.to_string())?;

    provider
        .persist_sync_output_artifact(source.as_path(), first_output.as_path())
        .await?;

    let second_output = temp.path().join("second-output");
    let response = provider
        .sync_document_extract_batch(
            source.to_string_lossy().as_ref(),
            second_output.to_string_lossy().as_ref(),
            false,
            false,
            "full",
        )
        .await?;

    assert_eq!(response.batches.len(), 1);
    assert!(second_output.join("_resources.arrow").exists());
    let mirrored = read_arrow_file(second_output.join("_resources.arrow").as_path())?;
    let resource_paths = mirrored[0]
        .column_by_name("resourcePath")
        .ok_or_else(|| "missing resourcePath column".to_string())?
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .ok_or_else(|| "resourcePath column is not Utf8".to_string())?;
    assert_eq!(
        resource_paths.value(0),
        second_output.join("scan.md").to_string_lossy()
    );
    Ok(())
}

#[test]
fn document_extract_success_resource_rows_are_cacheable() -> Result<(), String> {
    let batch = test_document_resource_batch("/tmp/source.png", "/tmp/output/source.md")?;

    assert!(document_extract_batches_are_cacheable(&[batch]));
    Ok(())
}

#[test]
fn document_extract_error_resource_rows_are_not_cacheable() -> Result<(), String> {
    let batch = arrow::record_batch::RecordBatch::try_new(
        Arc::new(arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("sourcePath", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("resourceType", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("resourcePath", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("pageIndex", arrow::datatypes::DataType::Int32, true),
            arrow::datatypes::Field::new("caption", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("content", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("mimeType", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("status", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("elementId", arrow::datatypes::DataType::Utf8, true),
        ])),
        vec![
            Arc::new(arrow::array::StringArray::from(vec!["/tmp/source.png"]))
                as arrow::array::ArrayRef,
            Arc::new(arrow::array::StringArray::from(vec!["error"])) as arrow::array::ArrayRef,
            Arc::new(arrow::array::StringArray::from(vec![""])) as arrow::array::ArrayRef,
            Arc::new(arrow::array::Int32Array::from(vec![-1])) as arrow::array::ArrayRef,
            Arc::new(arrow::array::StringArray::from(vec![""])) as arrow::array::ArrayRef,
            Arc::new(arrow::array::StringArray::from(vec!["conversion failed"]))
                as arrow::array::ArrayRef,
            Arc::new(arrow::array::StringArray::from(vec!["text/plain"])) as arrow::array::ArrayRef,
            Arc::new(arrow::array::StringArray::from(vec!["failed"])) as arrow::array::ArrayRef,
            Arc::new(arrow::array::StringArray::from(vec!["error"])) as arrow::array::ArrayRef,
        ],
    )
    .map_err(|error| error.to_string())?;

    assert!(!document_extract_batches_are_cacheable(&[batch]));
    Ok(())
}
