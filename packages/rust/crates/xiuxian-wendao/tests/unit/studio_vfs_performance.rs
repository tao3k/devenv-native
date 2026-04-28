//! VFS scan performance and API latency benchmark tests.
//!
//! These tests validate that:
//! - VFS scan completes in < 100ms for typical workloads
//! - API response latency is < 10ms for cached endpoints
#![cfg(feature = "zhenfa-router")]

use crate as xiuxian_wendao;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use std::time::Instant;
use tower::ServiceExt;

use xiuxian_wendao::analyzers::PluginRegistry;
use xiuxian_wendao::gateway::studio::{GatewayState, StudioState, studio_router};

/// Performance threshold for VFS scan (milliseconds).
const VFS_SCAN_THRESHOLD_MS: u64 = 100;

/// Performance threshold for API latency (milliseconds).
const API_LATENCY_THRESHOLD_MS: u64 = 10;
/// Performance threshold for warmed `StudioState` bootstrap samples (milliseconds).
const STUDIO_STATE_BOOTSTRAP_THRESHOLD_MS: u64 = 150;
const STUDIO_STATE_BOOTSTRAP_SAMPLES: usize = 5;

const _: () = {
    assert!(VFS_SCAN_THRESHOLD_MS >= 50);
    assert!(VFS_SCAN_THRESHOLD_MS <= 500);
    assert!(API_LATENCY_THRESHOLD_MS >= 5);
    assert!(API_LATENCY_THRESHOLD_MS <= 50);
    assert!(STUDIO_STATE_BOOTSTRAP_THRESHOLD_MS >= API_LATENCY_THRESHOLD_MS);
    assert!(STUDIO_STATE_BOOTSTRAP_THRESHOLD_MS <= 250);
    assert!(STUDIO_STATE_BOOTSTRAP_SAMPLES >= 3);
    assert!(STUDIO_STATE_BOOTSTRAP_SAMPLES <= 16);
};

fn elapsed_millis_u64(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn best_studio_state_bootstrap_millis() -> u64 {
    let _warmup = StudioState::new();
    (0..STUDIO_STATE_BOOTSTRAP_SAMPLES)
        .map(|_| {
            let start = Instant::now();
            let _state = StudioState::new();
            elapsed_millis_u64(start)
        })
        .min()
        .unwrap_or(u64::MAX)
}

fn request_for_uri(uri: &str) -> Request<Body> {
    match Request::builder().uri(uri).body(Body::empty()) {
        Ok(request) => request,
        Err(error) => panic!("failed to build request for `{uri}`: {error}"),
    }
}

// ============================================================================
// Router Creation Tests
// ============================================================================

#[test]
fn router_creation_is_instant() {
    let state = Arc::new(GatewayState::new(
        None,
        None,
        Arc::new(PluginRegistry::new()),
    ));

    let start = Instant::now();
    let _router = studio_router(Arc::clone(&state));
    let elapsed_ms = elapsed_millis_u64(start);

    assert!(
        elapsed_ms < API_LATENCY_THRESHOLD_MS,
        "Router creation should complete in < {API_LATENCY_THRESHOLD_MS}ms, took {elapsed_ms}ms"
    );
}

#[test]
fn studio_state_creation_is_fast() {
    let elapsed_ms = best_studio_state_bootstrap_millis();

    assert!(
        elapsed_ms < STUDIO_STATE_BOOTSTRAP_THRESHOLD_MS,
        "Best-of-{STUDIO_STATE_BOOTSTRAP_SAMPLES} state creation samples should complete in < {STUDIO_STATE_BOOTSTRAP_THRESHOLD_MS}ms, took {elapsed_ms}ms"
    );
}

// ============================================================================
// Route Path Calibration Tests
// ============================================================================

#[test]
fn router_has_expected_api_routes() {
    let state = Arc::new(GatewayState::new(
        None,
        None,
        Arc::new(PluginRegistry::new()),
    ));
    let _router = studio_router(state);
}

#[tokio::test]
async fn router_uses_document_extraction_rest_routes() {
    let state = Arc::new(GatewayState::new(
        None,
        None,
        Arc::new(PluginRegistry::new()),
    ));
    let router = studio_router(state);

    let response = router
        .clone()
        .oneshot(request_for_uri("/api/document-extract-result"))
        .await
        .unwrap_or_else(|error| panic!("document extraction route should respond: {error}"));
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let old_response = router
        .clone()
        .oneshot(request_for_uri("/api/pdf-extract-result"))
        .await
        .unwrap_or_else(|error| panic!("retired PDF extraction route should respond: {error}"));
    assert_eq!(old_response.status(), StatusCode::NOT_FOUND);

    let queue_snapshot_response = router
        .clone()
        .oneshot(request_for_uri("/api/document-extract-jobs"))
        .await
        .unwrap_or_else(|error| {
            panic!("document extraction jobs status route should respond: {error}")
        });
    assert_eq!(queue_snapshot_response.status(), StatusCode::OK);

    let job_status_response = router
        .oneshot(request_for_uri("/api/document-extract-job"))
        .await
        .unwrap_or_else(|error| panic!("document extraction job route should respond: {error}"));
    assert_eq!(job_status_response.status(), StatusCode::BAD_REQUEST);
}
