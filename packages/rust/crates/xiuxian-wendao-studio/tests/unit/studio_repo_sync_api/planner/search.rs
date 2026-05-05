use serde_json::Value;

use super::support::{hit_gap_matches_needle, modelica_nodocs_router};
use crate::studio::studio_repo_sync_api_tests::support::{
    create_local_git_repo, gateway_state_for_project, request_json, write_default_repo_config,
};
use crate::studio::studio_repo_sync_api_tests::{
    StatusCode, TestResult, assert_studio_json_snapshot, fs, studio_router,
};

#[tokio::test]
async fn docs_planner_search_endpoint_returns_gap_hits() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("orphan.md"), "# orphan\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/planner-search?repo=gateway-sync&query=orphan&page_kind=explanation&limit=5",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_planner_search_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_planner_search_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let (_temp_dir, router) = modelica_nodocs_router("modelica-gateway-sync")?;

    let (status, payload) = request_json(
        router,
        "/api/docs/planner-search?repo=modelica-gateway-sync&query=NoDocs&limit=4",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let hits = payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("planner-search payload should include a hits array")?;
    assert!(
        !hits.is_empty(),
        "planner-search endpoint should return at least one gap hit"
    );
    assert!(
        hits.len() <= 4,
        "planner-search endpoint should honor the configured hit limit"
    );
    assert!(
        hits.iter().all(|hit| hit_gap_matches_needle(hit, "NoDocs")),
        "planner-search endpoint hits should stay anchored to the injected no-doc target"
    );
    assert_studio_json_snapshot("docs_planner_search_endpoint_modelica_json", payload);
    Ok(())
}
