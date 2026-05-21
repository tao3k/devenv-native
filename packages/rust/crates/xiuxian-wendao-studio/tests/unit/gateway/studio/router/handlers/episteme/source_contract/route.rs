use std::time::Instant;

use axum::{
    body::{Body, to_bytes},
    http::StatusCode,
};
use tower::ServiceExt;

use crate::studio::router::studio_router;

use super::support::EpistemeGatewayFixture;
use super::{
    EPISTEME_EVIDENCE_READ_ROUTE, EPISTEME_EVIDENCE_SELECTION_PLAN_ROUTE,
    EPISTEME_SOURCE_CONTRACT_RUN_PLAN_ROUTE,
};

#[tokio::test]
async fn episteme_source_contract_gateway_route_writes_run_plan()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeGatewayFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;

    let body = serde_json::json!({
        "epistemeRoot": "source-contract",
        "corpusRoot": "corpus-root",
        "runId": "gateway_route_seed",
        "route": "document_text_evidence",
        "limit": 1
    });
    let request = axum::http::Request::builder()
        .method("POST")
        .uri(EPISTEME_SOURCE_CONTRACT_RUN_PLAN_ROUTE)
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))?;

    let response = studio_router(fixture.gateway_state())
        .oneshot(request)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        fixture
            .episteme_root
            .join("runs/extraction/gateway_route_seed/tasks.tsv")
            .is_file()
    );

    Ok(())
}

#[tokio::test]
async fn episteme_source_contract_gateway_selected_plan_hot_path_smoke()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeGatewayFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.add_source(
        "docs/b.docx",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_policy_category",
        "document_text_evidence",
        20,
    )?;
    fixture.write_contract()?;
    fixture.write_runtime_config()?;
    fixture.write_selection_run("selected_hot_seed", &["episteme.file.b"])?;

    let router = studio_router(fixture.gateway_state());
    let mut samples = Vec::new();
    for index in 0..12 {
        let body = serde_json::json!({
            "epistemeRoot": "source-contract",
            "selectionRunId": "selected_hot_seed",
            "runId": format!("gateway_hot_seed_{index}"),
            "limit": 12
        });
        let request = axum::http::Request::builder()
            .method("POST")
            .uri(EPISTEME_SOURCE_CONTRACT_RUN_PLAN_ROUTE)
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))?;

        let started = Instant::now();
        let response = router.clone().oneshot(request).await?;
        samples.push(started.elapsed().as_secs_f64() * 1000.0);

        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            fixture
                .episteme_root
                .join(format!(
                    "configured-runs/extraction/gateway_hot_seed_{index}/tasks.tsv"
                ))
                .is_file()
        );
    }
    samples.sort_by(f64::total_cmp);
    let p50 = samples[samples.len() / 2];
    let p95 = samples[samples.len() - 1];
    let max = samples[samples.len() - 1];
    println!(
        "episteme_gateway_selected_plan_hot_path_ms samples={} p50={p50:.3} p95={p95:.3} max={max:.3}",
        samples.len()
    );

    Ok(())
}

#[tokio::test]
async fn episteme_evidence_gateway_read_uses_episteme_toml_default()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeGatewayFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;
    fixture.write_runtime_config()?;

    let body = serde_json::json!({
        "epistemeRoot": "source-contract",
        "fileId": "episteme.file.a",
        "maxPreviewBytes": 12
    });
    let request = axum::http::Request::builder()
        .method("POST")
        .uri(EPISTEME_EVIDENCE_READ_ROUTE)
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))?;

    let response = studio_router(fixture.gateway_state())
        .oneshot(request)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let payload: serde_json::Value = serde_json::from_slice(&body)?;
    assert_eq!(payload["source"]["fileId"], "episteme.file.a");
    assert_eq!(payload["source"]["relativePath"], "docs/a.txt");
    assert_eq!(payload["previewKind"], "plain-text");
    assert_eq!(payload["textPreview"]["text"], "fixture cont");
    assert_eq!(payload["extractionExecuted"], false);
    assert_eq!(payload["rawToRdfPromotionAllowed"], false);
    assert_eq!(payload["validationMode"], "metadata-only");

    Ok(())
}

#[tokio::test]
async fn episteme_evidence_gateway_read_hot_path_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeGatewayFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;
    fixture.write_runtime_config()?;

    let router = studio_router(fixture.gateway_state());
    let mut samples = Vec::new();
    for _ in 0..12 {
        let body = serde_json::json!({
            "epistemeRoot": "source-contract",
            "fileId": "episteme.file.a",
            "maxPreviewBytes": 12
        });
        let request = axum::http::Request::builder()
            .method("POST")
            .uri(EPISTEME_EVIDENCE_READ_ROUTE)
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))?;

        let started = Instant::now();
        let response = router.clone().oneshot(request).await?;
        samples.push(started.elapsed().as_secs_f64() * 1000.0);

        assert_eq!(response.status(), StatusCode::OK);
    }
    samples.sort_by(f64::total_cmp);
    let p50 = samples[samples.len() / 2];
    let p95 = samples[samples.len() - 1];
    let max = samples[samples.len() - 1];
    println!(
        "episteme_gateway_evidence_read_hot_path_ms samples={} p50={p50:.3} p95={p95:.3} max={max:.3}",
        samples.len()
    );

    Ok(())
}

#[tokio::test]
async fn episteme_evidence_gateway_writes_selection_plan_from_episteme_toml()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeGatewayFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.add_source(
        "docs/b.txt",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_policy_category",
        "document_text_evidence",
        20,
    )?;
    fixture.write_contract()?;
    fixture.write_runtime_config()?;

    let body = serde_json::json!({
        "epistemeRoot": "source-contract",
        "runId": "gateway_selection_seed",
        "fileIds": ["episteme.file.b"],
        "selectionReason": "agent selected policy evidence"
    });
    let request = axum::http::Request::builder()
        .method("POST")
        .uri(EPISTEME_EVIDENCE_SELECTION_PLAN_ROUTE)
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))?;

    let response = studio_router(fixture.gateway_state())
        .oneshot(request)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        fixture
            .episteme_root
            .join("configured-runs/evidence-selection/gateway_selection_seed/selection.tsv")
            .is_file()
    );
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let payload: serde_json::Value = serde_json::from_slice(&body)?;
    assert_eq!(payload["runId"], "gateway_selection_seed");
    assert_eq!(payload["selectedCount"], 1);
    assert_eq!(payload["extractionExecuted"], false);
    assert_eq!(payload["rawToRdfPromotionAllowed"], false);
    assert_eq!(payload["validationMode"], "metadata-only");

    Ok(())
}

#[tokio::test]
async fn episteme_evidence_gateway_selection_plan_hot_path_smoke()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeGatewayFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;
    fixture.write_runtime_config()?;

    let router = studio_router(fixture.gateway_state());
    let mut samples = Vec::new();
    for index in 0..12 {
        let body = serde_json::json!({
            "epistemeRoot": "source-contract",
            "runId": format!("gateway_selection_hot_seed_{index}"),
            "fileIds": ["episteme.file.a"]
        });
        let request = axum::http::Request::builder()
            .method("POST")
            .uri(EPISTEME_EVIDENCE_SELECTION_PLAN_ROUTE)
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))?;

        let started = Instant::now();
        let response = router.clone().oneshot(request).await?;
        samples.push(started.elapsed().as_secs_f64() * 1000.0);

        assert_eq!(response.status(), StatusCode::OK);
    }
    samples.sort_by(f64::total_cmp);
    let p50 = samples[samples.len() / 2];
    let p95 = samples[samples.len() - 1];
    let max = samples[samples.len() - 1];
    println!(
        "episteme_gateway_selection_plan_hot_path_ms samples={} p50={p50:.3} p95={p95:.3} max={max:.3}",
        samples.len()
    );

    Ok(())
}
