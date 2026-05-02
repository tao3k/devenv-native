use serde_json::Value;

use super::support::{
    group_gaps_match_needle, group_preview_within_limit, modelica_nodocs_router, sum_u64_field,
};
use crate::gateway::studio::studio_repo_sync_api_tests::support::{
    create_local_git_repo, gateway_state_for_project, request_json, write_default_repo_config,
};
use crate::gateway::studio::studio_repo_sync_api_tests::{
    StatusCode, TestResult, assert_studio_json_snapshot, fs, studio_router,
};

#[tokio::test]
async fn docs_planner_queue_endpoint_returns_grouped_gap_backlog() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve, explain\nsolve() = nothing\nexplain() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("orphan_demo.jl"),
        "println(\"detached example\")\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("orphan.md"), "# orphan\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/planner-queue?repo=gateway-sync&per_kind_limit=2",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_planner_queue_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_planner_queue_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let (_temp_dir, router) = modelica_nodocs_router("modelica-gateway-queue")?;

    let (status, payload) = request_json(
        router,
        "/api/docs/planner-queue?repo=modelica-gateway-queue&gap_kind=symbol_reference_without_documentation&page_kind=reference&per_kind_limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let groups = payload
        .get("groups")
        .and_then(Value::as_array)
        .ok_or("planner-queue payload should include a groups array")?;
    let total_gap_count = payload
        .get("total_gap_count")
        .and_then(Value::as_u64)
        .ok_or("planner-queue payload should include total_gap_count")?;

    assert!(
        !groups.is_empty(),
        "planner-queue endpoint should return at least one grouped backlog lane"
    );
    assert_eq!(
        total_gap_count,
        sum_u64_field(groups, "count"),
        "planner-queue total should match grouped counts"
    );
    assert!(
        groups
            .iter()
            .all(|group| group_preview_within_limit(group, 3)),
        "planner-queue previews should honor per-kind truncation"
    );
    assert!(
        groups
            .iter()
            .all(|group| group_gaps_match_needle(group, "NoDocs")),
        "planner-queue endpoint gaps should stay anchored to the injected no-doc target"
    );
    assert_studio_json_snapshot("docs_planner_queue_endpoint_modelica_json", payload);
    Ok(())
}
