use serde_json::Value;

use crate::gateway::studio::studio_repo_sync_api_tests::support::{
    create_local_git_repo, create_local_modelica_repo, gateway_state_for_project, request_json,
    write_default_repo_config,
};
use crate::gateway::studio::studio_repo_sync_api_tests::{
    DocsProjectedGapReportQuery, StatusCode, TestResult, assert_studio_json_snapshot,
    docs_projected_gap_report_from_config, fs, studio_router,
};

#[tokio::test]
async fn docs_planner_item_endpoint_returns_gap_bundle() -> TestResult {
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
        "/api/docs/planner-item?repo=gateway-sync&gap_id=repo:gateway-sync:projection-gap:documentation_page_without_anchor:repo:gateway-sync:doc:docs/orphan.md&related_limit=3&family_limit=2",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_planner_item_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_planner_item_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        repo_dir.join("Controllers").join("NoDocs.mo"),
        "within Projectionica.Controllers;\nmodel NoDocs\nend NoDocs;\n",
    )?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-item]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let gap_report = docs_projected_gap_report_from_config(
        &DocsProjectedGapReportQuery {
            repo_id: "modelica-gateway-item".to_string(),
        },
        Some(&temp.path().join("wendao.toml")),
        temp.path(),
    )?;
    let gap = gap_report
        .gaps
        .first()
        .cloned()
        .ok_or("planner-item route expected at least one projected gap")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/docs/planner-item?repo=modelica-gateway-item&gap_id={}&family_kind=how_to&related_limit=3&family_limit=3",
            gap.gap_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let route_gap = payload
        .get("gap")
        .and_then(Value::as_object)
        .ok_or("planner-item payload should include a gap object")?;
    let route_gap_id = route_gap
        .get("gap_id")
        .and_then(Value::as_str)
        .ok_or("planner-item payload should include gap.gap_id")?;
    let route_page_id = route_gap
        .get("page_id")
        .and_then(Value::as_str)
        .ok_or("planner-item payload should include gap.page_id")?;
    let route_title = route_gap
        .get("title")
        .and_then(Value::as_str)
        .ok_or("planner-item payload should include gap.title")?;
    assert_eq!(
        route_gap_id, gap.gap_id,
        "planner-item route should reopen the requested stable gap"
    );
    assert!(
        route_title.contains("NoDocs") || route_page_id.contains("NoDocs"),
        "planner-item route should stay anchored to the injected no-doc target"
    );
    assert_eq!(
        payload
            .get("hit")
            .and_then(Value::as_object)
            .and_then(|hit| hit.get("page"))
            .and_then(Value::as_object)
            .and_then(|page| page.get("page_id"))
            .and_then(Value::as_str),
        Some(route_page_id),
        "planner-item route retrieval hit should stay anchored to the gap page"
    );
    assert_eq!(
        payload
            .get("navigation")
            .and_then(Value::as_object)
            .and_then(|navigation| navigation.get("center"))
            .and_then(Value::as_object)
            .and_then(|center| center.get("page"))
            .and_then(Value::as_object)
            .and_then(|page| page.get("page_id"))
            .and_then(Value::as_str),
        Some(route_page_id),
        "planner-item route navigation center should stay anchored to the gap page"
    );
    assert_studio_json_snapshot("docs_planner_item_endpoint_modelica_json", payload);
    Ok(())
}
