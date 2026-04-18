#![cfg(feature = "zhenfa-router")]
//! Router-level verification for the Studio search-plane status endpoint.

use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::util::ServiceExt;

use xiuxian_wendao::analyzers::PluginRegistry;
use xiuxian_wendao::gateway::studio::{GatewayState, studio_router};

type TestResult = Result<(), Box<dyn std::error::Error>>;

async fn request_json(
    router: axum::Router,
    uri: &str,
) -> Result<(StatusCode, Value), Box<dyn std::error::Error>> {
    let response = router
        .oneshot(Request::builder().uri(uri).body(Body::empty())?)
        .await?;
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let payload = serde_json::from_slice(&body)?;
    Ok((status, payload))
}

fn test_router() -> axum::Router {
    studio_router(Arc::new(GatewayState::new(
        None,
        None,
        Arc::new(PluginRegistry::new()),
    )))
}

fn assert_optional_repo_read_pressure(payload: &Value) {
    if let Some(repo_read_pressure) = payload
        .get("repoReadPressure")
        .filter(|value| !value.is_null())
    {
        assert!(repo_read_pressure["budget"].is_u64());
        assert!(repo_read_pressure["inFlight"].is_u64());
        assert!(
            repo_read_pressure
                .get("capturedAt")
                .is_none_or(|value| value.is_null() || value.is_string())
        );
        assert!(
            repo_read_pressure
                .get("requestedRepoCount")
                .is_none_or(|value| value.is_null() || value.is_u64())
        );
        assert!(
            repo_read_pressure
                .get("searchableRepoCount")
                .is_none_or(|value| value.is_null() || value.is_u64())
        );
        assert!(
            repo_read_pressure
                .get("parallelism")
                .is_none_or(|value| value.is_null() || value.is_u64())
        );
        assert!(repo_read_pressure["fanoutCapped"].is_boolean());
    }
}

fn assert_optional_status_reason(payload: &Value) {
    if let Some(status_reason) = payload.get("statusReason").filter(|value| !value.is_null()) {
        assert!(status_reason["code"].is_string());
        assert!(status_reason["severity"].is_string());
        assert!(status_reason["action"].is_string());
        assert!(status_reason["affectedCorpusCount"].is_u64());
        assert!(status_reason["readableCorpusCount"].is_u64());
        assert!(status_reason["blockingCorpusCount"].is_u64());
    }
}

fn corpus_phase_count(corpora: &[Value], phase: &str) -> u64 {
    corpora
        .iter()
        .filter(|entry| entry["phase"] == phase)
        .count() as u64
}

fn assert_idle_status_top_level(payload: &Value) -> TestResult {
    assert_eq!(payload["total"], Value::from(6));
    assert_eq!(payload["compactionPending"], Value::from(0));
    assert_eq!(
        payload["studioBootstrapBackgroundIndexingEnabled"],
        Value::from(false)
    );
    assert_eq!(
        payload["studioBootstrapBackgroundIndexingMode"],
        Value::from("deferred")
    );
    assert_eq!(
        payload["studioBootstrapBackgroundIndexingDeferredActivationObserved"],
        Value::from(false)
    );
    assert!(payload["studioBootstrapBackgroundIndexingDeferredActivationAt"].is_null());
    assert!(payload["studioBootstrapBackgroundIndexingDeferredActivationSource"].is_null());
    assert!(
        payload
            .get("queryTelemetrySummary")
            .is_none_or(Value::is_null)
    );
    assert_optional_repo_read_pressure(payload);
    assert_optional_status_reason(payload);

    let phase_total = payload["idle"].as_u64().ok_or("idle should be numeric")?
        + payload["indexing"]
            .as_u64()
            .ok_or("indexing should be numeric")?
        + payload["ready"].as_u64().ok_or("ready should be numeric")?
        + payload["degraded"]
            .as_u64()
            .ok_or("degraded should be numeric")?
        + payload["failed"]
            .as_u64()
            .ok_or("failed should be numeric")?;
    assert_eq!(phase_total, 6);
    Ok(())
}

fn assert_idle_status_corpora(payload: &Value) -> TestResult {
    let corpora = payload["corpora"]
        .as_array()
        .ok_or("corpora should be an array")?;
    assert_eq!(corpora.len(), 6);
    assert_eq!(
        payload["idle"],
        Value::from(corpus_phase_count(corpora, "idle"))
    );
    assert_eq!(
        payload["indexing"],
        Value::from(corpus_phase_count(corpora, "indexing"))
    );
    assert_eq!(
        payload["ready"],
        Value::from(corpus_phase_count(corpora, "ready"))
    );
    assert_eq!(
        payload["degraded"],
        Value::from(corpus_phase_count(corpora, "degraded"))
    );
    assert_eq!(
        payload["failed"],
        Value::from(corpus_phase_count(corpora, "failed"))
    );

    let local_symbol = corpora
        .iter()
        .find(|entry| entry["corpus"] == "local_symbol")
        .ok_or("local_symbol corpus should be present")?;
    assert!(
        matches!(local_symbol["phase"].as_str(), Some("idle" | "ready")),
        "local_symbol should stay cold-idle or restore a cached ready epoch: {local_symbol:?}"
    );
    let knowledge_section = corpora
        .iter()
        .find(|entry| entry["corpus"] == "knowledge_section")
        .ok_or("knowledge_section corpus should be present")?;
    assert!(
        matches!(knowledge_section["phase"].as_str(), Some("idle" | "ready")),
        "knowledge_section should stay cold-idle or restore a cached ready epoch: {knowledge_section:?}"
    );
    assert!(
        corpora
            .iter()
            .any(|entry| entry["corpus"] == "repo_content_chunk")
    );
    Ok(())
}

#[tokio::test]
async fn search_index_status_endpoint_returns_idle_corpora_snapshot() -> TestResult {
    let (status, payload) = request_json(test_router(), "/api/search/index/status").await?;
    assert_eq!(status, StatusCode::OK);
    assert_idle_status_top_level(&payload)?;
    assert_idle_status_corpora(&payload)?;
    Ok(())
}
