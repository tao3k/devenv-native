use super::support::spawn_vllm_sr_route_probe;
use std::sync::{Arc, Mutex};

use xiuxian_io::model_routing::VLLM_SR_AUTO_MODEL;
use xiuxian_wendao_server::transport::DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE;

use super::{
    DocumentExtractJobRegistry, DocumentExtractRouteSourceIdentity,
    ImageDocumentExtractRouteConfig, StudioDocumentExtractFlightRouteProvider, fs,
    image_document_extract_model_route_for_source_identity,
    image_document_extract_model_route_with_config, spawn_document_extract_service,
    test_document_resource_batch,
};

#[tokio::test]
async fn image_route_admission_uses_vllm_sr_decision() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("scan.PNG");
    fs::write(source.as_path(), b"image bytes").map_err(|error| error.to_string())?;
    let observed_payload = Arc::new(Mutex::new(None));
    let (vllm_sr_base_url, server_handle) =
        spawn_vllm_sr_route_probe(Arc::clone(&observed_payload)).await?;
    let config = ImageDocumentExtractRouteConfig {
        route_provider: Some("openrouter".to_owned()),
        route_model: "qwen/qwen3-vl-8b-instruct".to_owned(),
        model_routing_mode: xiuxian_io::model_routing::WendaoModelRoutingMode::VllmSr,
        vllm_sr_base_url,
    };

    let Some(route) = image_document_extract_model_route_with_config(
        source.as_path(),
        DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE,
        config,
    )
    .await?
    else {
        return Err("image vLLM-SR mode should produce a route decision".to_owned());
    };

    assert_eq!(route.intent.modality, "image");
    assert_eq!(route.intent.task_kind.as_str(), "attachment-extract");
    assert!(
        route
            .intent
            .artifact_refs
            .contains(&"source-suffix:png".to_owned())
    );
    assert_eq!(route.decision.selected_provider, "openrouter");
    assert_eq!(route.decision.selected_model, "qwen/qwen3-vl-8b-instruct");
    assert_eq!(
        route.decision.selected_backend_profile,
        DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE
    );
    let payload = observed_payload
        .lock()
        .map_err(|_| "observed payload lock poisoned".to_owned())?
        .clone()
        .ok_or_else(|| "vLLM-SR image route probe did not receive a request".to_owned())?;
    assert_eq!(
        payload.get("model").and_then(serde_json::Value::as_str),
        Some(VLLM_SR_AUTO_MODEL)
    );

    server_handle.abort();
    Ok(())
}

#[tokio::test]
async fn image_route_deterministic_mode_returns_gateway_decision() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("scan.PNG");
    fs::write(source.as_path(), b"image bytes").map_err(|error| error.to_string())?;
    let config = ImageDocumentExtractRouteConfig {
        route_provider: Some("openrouter".to_owned()),
        route_model: "qwen/qwen3-vl-8b-instruct".to_owned(),
        model_routing_mode: xiuxian_io::model_routing::WendaoModelRoutingMode::Deterministic,
        vllm_sr_base_url: "http://127.0.0.1:8888".to_owned(),
    };

    let route = image_document_extract_model_route_with_config(
        source.as_path(),
        DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE,
        config,
    )
    .await?
    .ok_or_else(|| "image deterministic mode should produce a route decision".to_owned())?;

    assert_eq!(route.intent.modality, "image");
    assert_eq!(
        route.decision.route_id,
        "deterministic:attachment-extract:image:openrouter:qwen-qwen3-vl-8b-instruct"
    );
    assert_eq!(route.decision.selected_provider, "openrouter");
    assert_eq!(route.decision.selected_model, "qwen/qwen3-vl-8b-instruct");
    assert_eq!(
        route.decision.selected_backend_profile,
        DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE
    );
    assert_eq!(
        route.model_routing_mode,
        xiuxian_io::model_routing::WendaoModelRoutingMode::Deterministic
    );
    Ok(())
}

#[tokio::test]
async fn image_route_uses_precomputed_source_hash_when_available() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("scan.PNG");
    fs::write(source.as_path(), b"image bytes").map_err(|error| error.to_string())?;
    let config = ImageDocumentExtractRouteConfig {
        route_provider: Some("openrouter".to_owned()),
        route_model: "qwen/qwen3-vl-8b-instruct".to_owned(),
        model_routing_mode: xiuxian_io::model_routing::WendaoModelRoutingMode::Deterministic,
        vllm_sr_base_url: "http://127.0.0.1:8888".to_owned(),
    };

    let route = image_document_extract_model_route_for_source_identity(
        DocumentExtractRouteSourceIdentity {
            path: source.as_path(),
            sha256: " precomputed-source-hash ",
        },
        DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE,
        config,
    )
    .await?
    .ok_or_else(|| "image route should use the precomputed source hash".to_owned())?;

    assert_eq!(route.source_sha256, "precomputed-source-hash");
    assert!(
        route
            .intent
            .artifact_refs
            .contains(&"source-sha256:precomputed-source-hash".to_owned())
    );
    Ok(())
}

#[tokio::test]
async fn image_route_metadata_is_forwarded_to_document_extract_flight() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("scan.png");
    fs::write(source.as_path(), b"image bytes").map_err(|error| error.to_string())?;
    let observed_payload = Arc::new(Mutex::new(None));
    let (vllm_sr_base_url, route_server_handle) =
        spawn_vllm_sr_route_probe(Arc::clone(&observed_payload)).await?;
    let config = ImageDocumentExtractRouteConfig {
        route_provider: Some("openrouter".to_owned()),
        route_model: "qwen/qwen3-vl-8b-instruct".to_owned(),
        model_routing_mode: xiuxian_io::model_routing::WendaoModelRoutingMode::VllmSr,
        vllm_sr_base_url,
    };
    let route = image_document_extract_model_route_with_config(
        source.as_path(),
        DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE,
        config,
    )
    .await?
    .ok_or_else(|| "image route decision was not produced".to_owned())?;

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

    let batches = provider
        .request_python_document_extract_with_model_route(
            source.to_string_lossy().as_ref(),
            output.to_string_lossy().as_ref(),
            true,
            false,
            DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE,
            Some(&route),
        )
        .await?;

    assert_eq!(batches.len(), 1);
    let observed = observed
        .lock()
        .map_err(|_| "observed request lock poisoned".to_owned())?
        .clone()
        .ok_or_else(|| "document extract Flight request was not observed".to_owned())?;
    assert_eq!(
        observed.descriptor_path,
        vec!["analysis".to_owned(), "document-extract".to_owned()]
    );
    assert_eq!(observed.route_id.as_deref(), Some("route-image-1"));
    assert_eq!(
        observed.route_task_kind.as_deref(),
        Some("attachment-extract")
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
    assert_eq!(observed.route_precision_tier.as_deref(), Some("high"));

    route_server_handle.abort();
    flight_server_handle.abort();
    Ok(())
}
