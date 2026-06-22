use super::{
    DocumentExtractJobRegistry, StudioDocumentExtractFlightRouteProvider,
    collect_document_extract_string_values, fs, test_document_resource_batch, write_arrow_file,
};

#[tokio::test]
async fn native_org_document_extract_preserves_org_and_attachment_links_without_docling()
-> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("ledger.org");
    fs::write(
        source.as_path(),
        concat!(
            "#+TITLE: Attachment Ledger\n",
            "[[file:missing.pdf][Missing source]]\n",
            "[[attachment:report.docx]]\n",
            "* Notes\n",
            "Native Org text.\n",
        ),
    )
    .map_err(|error| error.to_string())?;
    let registry = DocumentExtractJobRegistry::new(
        temp.path().join("jobs.duckdb"),
        temp.path().join("artifacts"),
    )?;
    let provider = StudioDocumentExtractFlightRouteProvider::from_registry(Ok(registry), 1);
    let output = temp.path().join("output");

    let response = provider
        .sync_document_extract_batch(
            source.to_string_lossy().as_ref(),
            output.to_string_lossy().as_ref(),
            true,
            false,
            "full",
        )
        .await?;

    let resource_types = collect_document_extract_string_values(&response.batches, "resourceType")?;
    let mime_types = collect_document_extract_string_values(&response.batches, "mimeType")?;
    let contents = collect_document_extract_string_values(&response.batches, "content")?;

    assert!(resource_types.iter().any(|value| value == "org-document"));
    assert_eq!(
        resource_types
            .iter()
            .filter(|value| value.as_str() == "org-attachment-link")
            .count(),
        2
    );
    assert!(mime_types.iter().any(|value| value == "text/org"));
    assert!(
        mime_types
            .iter()
            .any(|value| value == "application/vnd.xiuxian.org-attachment-link+json")
    );
    assert!(
        contents
            .iter()
            .any(|value| value.contains("\"targetPath\":\"missing.pdf\""))
    );
    assert!(output.join("_resources.arrow").exists());
    Ok(())
}

#[tokio::test]
async fn native_org_document_extract_marks_resolved_links_without_recursing_into_org()
-> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("ledger.org");
    let text_attachment = temp.path().join("notes.txt");
    let org_attachment = temp.path().join("nested.org");
    fs::write(text_attachment.as_path(), "plain text attachment")
        .map_err(|error| error.to_string())?;
    fs::write(org_attachment.as_path(), "* Nested ledger\n").map_err(|error| error.to_string())?;
    fs::write(
        source.as_path(),
        concat!(
            "#+TITLE: Attachment Ledger\n",
            "[[file:notes.txt][Plain notes]]\n",
            "[[file:nested.org][Nested Org]]\n",
        ),
    )
    .map_err(|error| error.to_string())?;
    let registry = DocumentExtractJobRegistry::new(
        temp.path().join("jobs.duckdb"),
        temp.path().join("artifacts"),
    )?;
    let provider = StudioDocumentExtractFlightRouteProvider::from_registry(Ok(registry), 1);
    let output = temp.path().join("output");

    let response = provider
        .sync_document_extract_batch(
            source.to_string_lossy().as_ref(),
            output.to_string_lossy().as_ref(),
            true,
            false,
            "full",
        )
        .await?;

    let resource_types = collect_document_extract_string_values(&response.batches, "resourceType")?;
    let contents = collect_document_extract_string_values(&response.batches, "content")?;
    let attachment_rows = contents
        .iter()
        .filter(|value| value.contains("xiuxian_wendao.org_attachment_link.v1"))
        .collect::<Vec<_>>();

    assert_eq!(
        resource_types
            .iter()
            .filter(|value| value.as_str() == "org-attachment-link")
            .count(),
        2
    );
    assert_eq!(attachment_rows.len(), 2);
    assert!(
        attachment_rows
            .iter()
            .any(|value| value.contains("\"targetPath\":\"notes.txt\"")
                && value.contains("\"resolved\":true")
                && value.contains("\"analyzerEligible\":false"))
    );
    assert!(
        attachment_rows
            .iter()
            .any(|value| value.contains("\"targetPath\":\"nested.org\"")
                && value.contains("\"resolved\":true")
                && value.contains("\"analyzerEligible\":false"))
    );
    assert_eq!(response.batches.len(), 3);
    Ok(())
}

#[tokio::test]
async fn native_org_document_extract_bypasses_stale_generic_cache() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("ledger.org");
    fs::write(source.as_path(), "* Native ledger\n").map_err(|error| error.to_string())?;
    let registry = DocumentExtractJobRegistry::new(
        temp.path().join("jobs.duckdb"),
        temp.path().join("artifacts"),
    )?;
    let provider = StudioDocumentExtractFlightRouteProvider::from_registry(Ok(registry), 1);
    let output = temp.path().join("output");
    fs::create_dir_all(output.as_path()).map_err(|error| error.to_string())?;
    let stale_markdown = output.join("ledger.md");
    fs::write(stale_markdown.as_path(), "# stale Docling output\n")
        .map_err(|error| error.to_string())?;
    let stale_batch = test_document_resource_batch(
        source.to_string_lossy().as_ref(),
        stale_markdown.to_string_lossy().as_ref(),
    )?;
    write_arrow_file(
        output.join("_resources.arrow").as_path(),
        std::slice::from_ref(&stale_batch),
    )?;
    fs::write(output.join("_complete.marker"), b"").map_err(|error| error.to_string())?;

    let response = provider
        .sync_document_extract_batch(
            source.to_string_lossy().as_ref(),
            output.to_string_lossy().as_ref(),
            false,
            false,
            "full",
        )
        .await?;

    let resource_types = collect_document_extract_string_values(&response.batches, "resourceType")?;
    let element_ids = collect_document_extract_string_values(&response.batches, "elementId")?;
    assert_eq!(resource_types, vec!["org-document"]);
    assert_eq!(element_ids, vec!["_org_document"]);
    Ok(())
}
