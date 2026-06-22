use serde_json::Value;

use crate::studio::studio_repo_sync_api_tests::support::{
    create_local_git_repo, create_local_modelica_repo, gateway_state_for_project, request_json,
    write_default_repo_config, write_modelica_repo_config,
};
use crate::studio::studio_repo_sync_api_tests::{
    ProjectionPageKind, RepoProjectedPagesQuery, StatusCode, TestResult,
    assert_studio_json_snapshot, fs, repo_projected_pages_from_config, studio_router,
};

#[tokio::test]
async fn repo_projected_page_endpoint_returns_projection_payload() -> TestResult {
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
        "/api/repo/projected-page?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_page_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[sources.projects.modelica-gateway-projected-page]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-projected-page".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "Projectionica.Controllers.PI"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `Projectionica.Controllers.PI`"
            )
        });
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-page?repo=modelica-gateway-projected-page&page_id={}",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("page")
            .and_then(Value::as_object)
            .and_then(|page_value| page_value.get("page_id"))
            .and_then(Value::as_str)
            .is_some_and(|page_id| page_id == page.page_id),
        "repo-projected-page endpoint should stay anchored to the requested Modelica projected page"
    );
    assert_studio_json_snapshot("repo_projected_page_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_search_endpoint_returns_projection_payload() -> TestResult {
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
        "/api/repo/projected-page-search?repo=gateway-sync&query=solve&kind=reference&limit=5",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_page_search_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_search_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(
        temp.path(),
        &repo_dir,
        "modelica-gateway-projected-page-search",
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-search?repo=modelica-gateway-projected-page-search&query=Projectionica.Controllers&kind=reference&limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let pages = payload
        .get("pages")
        .and_then(Value::as_array)
        .ok_or("repo-projected-page-search payload should include a pages array")?;
    assert!(
        !pages.is_empty(),
        "repo-projected-page-search endpoint should return at least one projected page over the external Modelica path"
    );
    assert!(
        pages.len() <= 3,
        "repo-projected-page-search endpoint should honor the configured page limit"
    );
    assert!(
        pages.iter().any(|page| {
            page.get("title")
                .and_then(Value::as_str)
                .is_some_and(|title| title.contains("Projectionica.Controllers"))
        }),
        "repo-projected-page-search endpoint should keep page hits anchored to the requested Modelica controller path"
    );
    assert_studio_json_snapshot("repo_projected_page_search_endpoint_modelica_json", payload);
    Ok(())
}
