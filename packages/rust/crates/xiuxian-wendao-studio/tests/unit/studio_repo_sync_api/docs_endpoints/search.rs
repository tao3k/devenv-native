use serde_json::Value;

use crate::studio::studio_repo_sync_api_tests::support::{
    create_local_git_repo, create_local_modelica_repo, gateway_state_for_project,
    page_matches_needle, request_json, write_default_repo_config,
};
use crate::studio::studio_repo_sync_api_tests::{
    StatusCode, TestResult, assert_studio_json_snapshot, fs, studio_router,
};

#[tokio::test]
async fn docs_search_endpoint_returns_projection_payload() -> TestResult {
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
        "/api/docs/search?repo=gateway-sync&query=solve&kind=reference&limit=5",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_search_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_search_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[sources.projects.modelica-gateway-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/search?repo=modelica-gateway-search&query=Projectionica.Controllers&kind=reference&limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let pages = payload
        .get("pages")
        .and_then(Value::as_array)
        .ok_or("docs-search payload should include a pages array")?;
    assert!(
        !pages.is_empty(),
        "docs-search endpoint should return at least one projected page"
    );
    assert!(
        pages.len() <= 3,
        "docs-search endpoint should honor the configured hit limit"
    );
    assert!(
        pages.iter().all(|page| {
            page.as_object()
                .is_some_and(|page| page_matches_needle(page, "Projectionica.Controllers"))
        }),
        "docs-search endpoint pages should stay anchored to the requested Modelica controller path"
    );
    assert_studio_json_snapshot("docs_search_endpoint_modelica_json", payload);
    Ok(())
}
