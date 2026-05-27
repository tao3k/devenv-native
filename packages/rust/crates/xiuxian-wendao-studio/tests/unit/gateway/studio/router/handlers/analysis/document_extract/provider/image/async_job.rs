use std::sync::{Arc, Mutex};

use xiuxian_wendao_server::transport::DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE;

use super::{
    DocumentExtractJobRegistry, StudioDocumentExtractFlightRouteProvider, fs,
    spawn_document_extract_service, test_document_resource_batch,
};

#[tokio::test]
async fn async_image_job_forwards_route_metadata_and_writes_manifest() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("scan.png");
    fs::write(source.as_path(), b"image bytes").map_err(|error| error.to_string())?;
    let output = temp.path().join("out");
    let markdown = output.join("scan.md");
    let response_batch = test_document_resource_batch(
        source.to_string_lossy().as_ref(),
        markdown.to_string_lossy().as_ref(),
    )?;
    let observed = Arc::new(Mutex::new(None));
    let (endpoint, flight_server_handle) =
        spawn_document_extract_service(response_batch, Arc::clone(&observed)).await?;
    let registry = DocumentExtractJobRegistry::new(
        temp.path().join("jobs.duckdb"),
        temp.path().join("artifacts"),
    )?;
    let provider =
        StudioDocumentExtractFlightRouteProvider::from_registry_with_document_extract_endpoint(
            Ok(registry),
            1,
            endpoint,
        );
    let queued = provider
        .registry()?
        .submit(source.as_path(), output.as_path(), true)?;

    provider.run_job(queued.job_id.as_str()).await?;

    let status = provider
        .status(queued.job_id.as_str())?
        .ok_or_else(|| "image job should remain registered".to_owned())?;
    assert_eq!(status.status, "succeeded");
    let observed = observed
        .lock()
        .map_err(|_| "observed request lock poisoned".to_owned())?
        .clone()
        .ok_or_else(|| "document extract Flight request was not observed".to_owned())?;
    assert_eq!(
        observed.profile.as_deref(),
        Some(DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE)
    );
    assert_eq!(observed.route_modality.as_deref(), Some("image"));
    assert_eq!(
        observed.route_selected_provider.as_deref(),
        Some("openrouter")
    );
    assert_eq!(
        observed.route_selected_model.as_deref(),
        Some("qwen/qwen3-vl-8b-instruct")
    );
    assert_eq!(
        observed.route_selected_backend_profile.as_deref(),
        Some(DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE)
    );

    let manifest = output.join("_document_extract_model_route_manifest.json");
    assert!(
        manifest.exists(),
        "async image job should mirror the model route manifest into output"
    );
    let manifest_value = serde_json::from_slice::<serde_json::Value>(
        fs::read(manifest.as_path())
            .map_err(|error| error.to_string())?
            .as_slice(),
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        manifest_value["routeSelectedBackendProfile"],
        DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE
    );

    flight_server_handle.abort();
    Ok(())
}
