use serde_json::Value;

use crate::studio::studio_repo_sync_api_tests::support::{
    create_local_git_repo, create_local_modelica_repo, gateway_state_for_project, request_json,
    write_default_repo_config, write_modelica_repo_config,
};
use crate::studio::studio_repo_sync_api_tests::{
    StatusCode, TestResult, assert_studio_json_snapshot, fs, studio_router,
};

fn assert_count_matches_len(count: u64, len: usize, message: &str) {
    assert_eq!(usize::try_from(count).ok(), Some(len), "{message}");
}

fn projected_gap_report_router()
-> Result<(tempfile::TempDir, axum::Router), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));
    Ok((temp, router))
}

fn modelica_projected_gap_report_router(
    repo_id: &str,
) -> Result<(tempfile::TempDir, axum::Router), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(temp.path(), &repo_dir, repo_id)?;
    let router = studio_router(gateway_state_for_project(temp.path()));
    Ok((temp, router))
}

#[tokio::test]
async fn repo_projected_gap_report_endpoint_returns_projection_gap_payload() -> TestResult {
    let (_temp_dir, router) = projected_gap_report_router()?;

    let (status, payload) =
        request_json(router, "/api/repo/projected-gap-report?repo=gateway-sync").await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_gap_report_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_gap_report_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let (_temp_dir, router) =
        modelica_projected_gap_report_router("modelica-gateway-projected-gap-report")?;

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-gap-report?repo=modelica-gateway-projected-gap-report",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("repo_id").and_then(Value::as_str),
        Some("modelica-gateway-projected-gap-report")
    );
    let gaps = payload
        .get("gaps")
        .and_then(Value::as_array)
        .ok_or("repo-projected-gap-report payload should include a gaps array")?;
    let summary = payload
        .get("summary")
        .and_then(Value::as_object)
        .ok_or("repo-projected-gap-report payload should include a summary object")?;
    let gap_count = summary
        .get("gap_count")
        .and_then(Value::as_u64)
        .ok_or("repo-projected-gap-report summary should include gap_count")?;
    let page_count = summary
        .get("page_count")
        .and_then(Value::as_u64)
        .ok_or("repo-projected-gap-report summary should include page_count")?;
    assert_count_matches_len(
        gap_count,
        gaps.len(),
        "repo-projected-gap-report summary should stay aligned with the materialized gap list",
    );
    assert!(
        page_count > 0,
        "repo-projected-gap-report summary should reflect non-empty projected pages over the external Modelica path"
    );
    assert_studio_json_snapshot("repo_projected_gap_report_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_projected_gap_report_endpoint_returns_projection_gap_payload() -> TestResult {
    let (_temp_dir, router) = projected_gap_report_router()?;

    let (status, payload) =
        request_json(router, "/api/docs/projected-gap-report?repo=gateway-sync").await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_projected_gap_report_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_projected_gap_report_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let (_temp_dir, router) =
        modelica_projected_gap_report_router("modelica-gateway-projected-gap-report")?;

    let (status, payload) = request_json(
        router,
        "/api/docs/projected-gap-report?repo=modelica-gateway-projected-gap-report",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let gaps = payload
        .get("gaps")
        .and_then(Value::as_array)
        .ok_or("docs-projected-gap-report payload should include a gaps array")?;
    let summary_gap_count = payload
        .get("summary")
        .and_then(Value::as_object)
        .and_then(|summary| summary.get("gap_count"))
        .and_then(Value::as_u64)
        .ok_or("docs-projected-gap-report payload should include summary.gap_count")?;
    assert_count_matches_len(
        summary_gap_count,
        gaps.len(),
        "docs-projected-gap-report endpoint should keep summary.gap_count aligned with the materialized gap list",
    );
    assert!(
        payload
            .get("repo_id")
            .and_then(Value::as_str)
            .is_some_and(|repo_id| repo_id == "modelica-gateway-projected-gap-report"),
        "docs-projected-gap-report endpoint should stay anchored to the requested external Modelica repo"
    );
    assert_studio_json_snapshot("docs_projected_gap_report_endpoint_modelica_json", payload);
    Ok(())
}
