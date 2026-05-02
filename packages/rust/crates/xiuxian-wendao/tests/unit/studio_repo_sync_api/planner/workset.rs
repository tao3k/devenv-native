use serde_json::Value;

use super::support::{
    hit_gap_matches_needle, modelica_nodocs_router, selected_count_sum, sum_u64_field,
};
use crate::gateway::studio::studio_repo_sync_api_tests::support::{
    create_local_git_repo, gateway_state_for_project, request_json, write_default_repo_config,
};
use crate::gateway::studio::studio_repo_sync_api_tests::{
    StatusCode, TestResult, assert_studio_json_snapshot, fs, studio_router,
};

#[tokio::test]
async fn docs_planner_workset_endpoint_returns_opened_gap_batch() -> TestResult {
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
        "/api/docs/planner-workset?repo=gateway-sync&per_kind_limit=2&limit=2&family_limit=2",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_planner_workset_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_planner_workset_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let (_temp_dir, router) = modelica_nodocs_router("modelica-gateway-workset")?;

    let (status, payload) = request_json(
        router,
        "/api/docs/planner-workset?repo=modelica-gateway-workset&gap_kind=symbol_reference_without_documentation&page_kind=reference&per_kind_limit=3&limit=4&family_kind=how_to&related_limit=3&family_limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let items = payload
        .get("items")
        .and_then(Value::as_array)
        .ok_or("planner-workset payload should include an items array")?;
    let ranked_hits = payload
        .get("ranked_hits")
        .and_then(Value::as_array)
        .ok_or("planner-workset payload should include a ranked_hits array")?;
    let queue = payload
        .get("queue")
        .and_then(Value::as_object)
        .ok_or("planner-workset payload should include a queue object")?;
    let queue_groups = queue
        .get("groups")
        .and_then(Value::as_array)
        .ok_or("planner-workset payload should include queue.groups")?;
    let total_gap_count = queue
        .get("total_gap_count")
        .and_then(Value::as_u64)
        .ok_or("planner-workset payload should include queue.total_gap_count")?;
    let groups = payload
        .get("groups")
        .and_then(Value::as_array)
        .ok_or("planner-workset payload should include groups")?;

    assert!(
        !items.is_empty(),
        "planner-workset endpoint should select at least one Modelica workset item"
    );
    assert_eq!(
        items.len(),
        ranked_hits.len(),
        "planner-workset endpoint should reopen every ranked hit into one item"
    );
    assert!(
        items.len() <= 4,
        "planner-workset endpoint should honor the ranked-hit limit"
    );
    assert_eq!(
        total_gap_count,
        sum_u64_field(queue_groups, "count"),
        "planner-workset queue total should match grouped counts"
    );
    assert_eq!(
        selected_count_sum(groups),
        Some(items.len()),
        "planner-workset grouped selected counts should match opened items"
    );
    assert!(
        items
            .iter()
            .all(|item| hit_gap_matches_needle(item, "NoDocs")),
        "planner-workset endpoint items should stay anchored to the injected no-doc target"
    );
    assert_studio_json_snapshot("docs_planner_workset_endpoint_modelica_json", payload);
    Ok(())
}
