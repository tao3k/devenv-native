use std::sync::{Arc, Mutex};

use axum::http::{HeaderMap, HeaderValue};
use axum::{Json, Router, routing::post};
use serde_json::json;
use tokio::net::TcpListener;
use xiuxian_llm::model_routing::{
    VLLM_SR_AUTO_MODEL, VLLM_SR_REQUEST_ID_HEADER, VLLM_SR_SELECTED_DECISION_HEADER,
    VLLM_SR_SELECTED_MODEL_HEADER,
};
use xiuxian_wendao_server::transport::{
    DOCUMENT_EXTRACT_FAST_TEXT_PROFILE, DOCUMENT_EXTRACT_FULL_PROFILE,
    DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE, DocumentExtractMode,
};

use super::runtime::flight_support::spawn_document_extract_service;
use super::{
    DocumentExtractJobRegistry, ImageDocumentExtractRouteConfig,
    StudioDocumentExtractFlightRouteProvider, fs, gateway_document_extract_mode_for_source,
    gateway_document_extract_profile_for_source, image_document_extract_model_route_with_config,
    test_document_resource_batch,
};

#[test]
fn auto_mode_keeps_image_extraction_on_sync_route() {
    assert_eq!(
        gateway_document_extract_mode_for_source("/tmp/scan.PNG"),
        DocumentExtractMode::Sync
    );
}

#[test]
fn full_profile_image_source_uses_hosted_vlm_image_profile() {
    assert_eq!(
        gateway_document_extract_profile_for_source("/tmp/scan.PNG", DOCUMENT_EXTRACT_FULL_PROFILE),
        DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE
    );
}

#[test]
fn explicit_non_full_profile_is_not_rewritten_for_image_source() {
    assert_eq!(
        gateway_document_extract_profile_for_source(
            "/tmp/scan.png",
            DOCUMENT_EXTRACT_FAST_TEXT_PROFILE,
        ),
        DOCUMENT_EXTRACT_FAST_TEXT_PROFILE
    );
}

#[test]
fn non_image_full_profile_stays_full() {
    assert_eq!(
        gateway_document_extract_profile_for_source(
            "/tmp/report.pdf",
            DOCUMENT_EXTRACT_FULL_PROFILE
        ),
        DOCUMENT_EXTRACT_FULL_PROFILE
    );
}

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
        model_routing_mode: xiuxian_llm::model_routing::WendaoModelRoutingMode::VllmSr,
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
async fn image_route_metadata_is_forwarded_to_document_extract_flight() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("scan.png");
    fs::write(source.as_path(), b"image bytes").map_err(|error| error.to_string())?;
    let observed_payload = Arc::new(Mutex::new(None));
    let (vllm_sr_base_url, route_server_handle) =
        spawn_vllm_sr_route_probe(Arc::clone(&observed_payload)).await?;
    let config = ImageDocumentExtractRouteConfig {
        route_provider: Some("openrouter".to_owned()),
        model_routing_mode: xiuxian_llm::model_routing::WendaoModelRoutingMode::VllmSr,
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

async fn spawn_vllm_sr_route_probe(
    observed_payload: Arc<Mutex<Option<serde_json::Value>>>,
) -> Result<(String, tokio::task::JoinHandle<()>), String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| format!("bind vLLM-SR route probe: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("read vLLM-SR route probe address: {error}"))?;
    let app = Router::new().route(
        "/v1/chat/completions",
        post(move |Json(payload): Json<serde_json::Value>| {
            let observed_payload = Arc::clone(&observed_payload);
            async move {
                *observed_payload
                    .lock()
                    .expect("observed payload lock should not be poisoned") = Some(payload);
                let mut headers = HeaderMap::new();
                headers.insert(
                    VLLM_SR_SELECTED_MODEL_HEADER,
                    HeaderValue::from_static("qwen/qwen3-vl-8b-instruct"),
                );
                headers.insert(
                    VLLM_SR_SELECTED_DECISION_HEADER,
                    HeaderValue::from_static("image-document-vlm"),
                );
                headers.insert(
                    VLLM_SR_REQUEST_ID_HEADER,
                    HeaderValue::from_static("route-image-1"),
                );
                (headers, Json(json!({"model": "qwen/qwen3-vl-8b-instruct"})))
            }
        }),
    );
    let handle = tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            panic!("vLLM-SR route probe failed: {error}");
        }
    });
    Ok((format!("http://{address}"), handle))
}
