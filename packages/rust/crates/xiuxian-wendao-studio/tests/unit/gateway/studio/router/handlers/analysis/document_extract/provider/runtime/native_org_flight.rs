use std::sync::{Arc, Mutex};

use xiuxian_wendao_server::transport::ANALYSIS_DOCUMENT_EXTRACT_ROUTE;

use super::{
    DocumentExtractJobRegistry, StudioDocumentExtractFlightRouteProvider,
    collect_document_extract_string_values,
    flight_support::{
        spawn_document_extract_service, spawn_document_extract_service_with_observed_requests,
    },
    fs, test_document_resource_batch,
};

#[tokio::test]
async fn native_org_document_extract_merges_resolved_eligible_attachment_from_analyzer_flight()
-> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("ledger.org");
    let markdown_attachment = temp.path().join("attachment.md");
    fs::write(
        markdown_attachment.as_path(),
        "# Attachment\n\nExtractable content.",
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        source.as_path(),
        concat!(
            "#+TITLE: Attachment Ledger\n",
            "[[file:attachment.md][Markdown attachment]]\n",
        ),
    )
    .map_err(|error| error.to_string())?;

    let analyzer_resource = temp.path().join("attachment-output.md");
    let response_batch = test_document_resource_batch(
        markdown_attachment.to_string_lossy().as_ref(),
        analyzer_resource.to_string_lossy().as_ref(),
    )?;
    let observed = Arc::new(Mutex::new(None));
    let (endpoint, server_handle) =
        spawn_document_extract_service(response_batch, Arc::clone(&observed)).await?;

    let registry = DocumentExtractJobRegistry::new(
        temp.path().join("jobs.duckdb"),
        temp.path().join("artifacts"),
    )?;
    let provider =
        StudioDocumentExtractFlightRouteProvider::from_registry_with_document_extract_endpoint(
            Ok(registry),
            1,
            endpoint.as_str(),
        );
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
    server_handle.abort();

    let resource_types = collect_document_extract_string_values(&response.batches, "resourceType")?;
    let source_paths = collect_document_extract_string_values(&response.batches, "sourcePath")?;
    assert!(resource_types.iter().any(|value| value == "org-document"));
    assert!(
        resource_types
            .iter()
            .any(|value| value == "org-attachment-link")
    );
    assert!(resource_types.iter().any(|value| value == "document"));
    assert!(
        source_paths
            .iter()
            .any(|value| value == markdown_attachment.to_string_lossy().as_ref())
    );

    let observed_request = observed
        .lock()
        .map_err(|_| "observed request lock poisoned".to_owned())?
        .clone()
        .ok_or_else(|| "document extract Flight request was not observed".to_owned())?;
    assert_eq!(
        observed_request.descriptor_path,
        ANALYSIS_DOCUMENT_EXTRACT_ROUTE
            .trim_start_matches('/')
            .split('/')
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        observed_request.source_path.as_deref(),
        Some(markdown_attachment.to_string_lossy().as_ref())
    );
    assert_eq!(observed_request.profile.as_deref(), Some("full"));
    assert!(
        observed_request
            .output_dir
            .as_deref()
            .is_some_and(|value| value.contains("_org_attachments")),
        "attachment analyzer output dir should stay under the Org output namespace: {observed_request:?}"
    );
    Ok(())
}

#[tokio::test]
async fn native_org_document_extract_merges_multiple_attachment_results_in_source_order()
-> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("ledger.org");
    let first_attachment = temp.path().join("first.md");
    let second_attachment = temp.path().join("second.md");
    fs::write(first_attachment.as_path(), "# First\n").map_err(|error| error.to_string())?;
    fs::write(second_attachment.as_path(), "# Second\n").map_err(|error| error.to_string())?;
    fs::write(
        source.as_path(),
        concat!(
            "#+TITLE: Attachment Ledger\n",
            "[[file:first.md][First attachment]]\n",
            "[[file:second.md][Second attachment]]\n",
        ),
    )
    .map_err(|error| error.to_string())?;

    let analyzer_resource = temp.path().join("attachment-output.md");
    let response_batch = test_document_resource_batch(
        first_attachment.to_string_lossy().as_ref(),
        analyzer_resource.to_string_lossy().as_ref(),
    )?;
    let observed = Arc::new(Mutex::new(None));
    let observed_requests = Arc::new(Mutex::new(Vec::new()));
    let (endpoint, server_handle) = spawn_document_extract_service_with_observed_requests(
        response_batch,
        Arc::clone(&observed),
        Arc::clone(&observed_requests),
    )
    .await?;

    let registry = DocumentExtractJobRegistry::new(
        temp.path().join("jobs.duckdb"),
        temp.path().join("artifacts"),
    )?;
    let provider =
        StudioDocumentExtractFlightRouteProvider::from_registry_with_document_extract_endpoint(
            Ok(registry),
            2,
            endpoint.as_str(),
        );
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
    server_handle.abort();

    let resource_types = collect_document_extract_string_values(&response.batches, "resourceType")?;
    assert_eq!(
        resource_types,
        vec![
            "org-document",
            "org-attachment-link",
            "document",
            "org-attachment-link",
            "document",
        ]
    );
    let element_ids = collect_document_extract_string_values(&response.batches, "elementId")?;
    assert!(element_ids.iter().any(|value| value == "_org_document"));
    assert!(
        element_ids
            .iter()
            .any(|value| value == "_org_attachment_0000_main")
    );
    assert!(
        element_ids
            .iter()
            .any(|value| value == "_org_attachment_0001_main")
    );
    let observed_requests = observed_requests
        .lock()
        .map_err(|_| "observed request sequence lock poisoned".to_owned())?
        .clone();
    assert_eq!(observed_requests.len(), 2);
    let observed_source_paths = observed_requests
        .iter()
        .filter_map(|request| request.source_path.as_deref())
        .collect::<Vec<_>>();
    assert!(observed_source_paths.contains(&first_attachment.to_string_lossy().as_ref()));
    assert!(observed_source_paths.contains(&second_attachment.to_string_lossy().as_ref()));
    Ok(())
}
