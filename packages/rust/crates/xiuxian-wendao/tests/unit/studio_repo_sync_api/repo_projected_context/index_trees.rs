use serde_json::Value;

use crate::gateway::studio::studio_repo_sync_api_tests::support::{
    create_local_git_repo, create_local_modelica_repo, gateway_state_for_project,
    projected_page_id_for_title, request_json, write_default_repo_config,
    write_modelica_repo_config,
};
use crate::gateway::studio::studio_repo_sync_api_tests::{
    ProjectionPageKind, StatusCode, TestResult, assert_studio_json_snapshot, fs, studio_router,
};

#[tokio::test]
async fn repo_projected_page_index_trees_endpoint_returns_tree_payload() -> TestResult {
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

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-index-trees?repo=gateway-sync",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_page_index_trees_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_trees_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(
        temp.path(),
        &repo_dir,
        "modelica-gateway-projected-index-trees",
    )?;
    let selected_page_id = projected_page_id_for_title(
        temp.path(),
        "modelica-gateway-projected-index-trees",
        ProjectionPageKind::Reference,
        "Projectionica.Controllers.PI",
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-index-trees?repo=modelica-gateway-projected-index-trees",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("repo_id").and_then(Value::as_str),
        Some("modelica-gateway-projected-index-trees")
    );
    let trees = payload
        .get("trees")
        .and_then(Value::as_array)
        .ok_or("repo-projected-page-index-trees payload should include a trees array")?;
    assert!(
        !trees.is_empty(),
        "repo-projected-page-index-trees endpoint should return at least one projected tree over the external Modelica path"
    );
    assert!(
        trees.iter().any(|tree| {
            tree.get("page_id")
                .and_then(Value::as_str)
                .is_some_and(|page_id| page_id == selected_page_id)
        }),
        "repo-projected-page-index-trees endpoint should include the projected tree for the selected Modelica symbol page"
    );
    assert_studio_json_snapshot(
        "repo_projected_page_index_trees_endpoint_modelica_json",
        payload,
    );
    Ok(())
}
