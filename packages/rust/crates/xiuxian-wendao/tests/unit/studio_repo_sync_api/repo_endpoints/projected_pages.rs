use super::super::*;

#[tokio::test]
async fn repo_projected_pages_endpoint_returns_projection_payload() -> TestResult {
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

    let (status, payload) =
        request_json(router, "/api/repo/projected-pages?repo=gateway-sync").await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_pages_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_pages_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-projected-pages]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-pages?repo=modelica-gateway-projected-pages",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let pages = payload
        .get("pages")
        .and_then(Value::as_array)
        .ok_or("repo-projected-pages payload should include a pages array")?;
    assert!(
        !pages.is_empty(),
        "repo-projected-pages endpoint should return at least one projected page over the external Modelica path"
    );
    assert!(
        pages.iter().any(|page| {
            let title = page
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let page_id = page
                .get("page_id")
                .and_then(Value::as_str)
                .unwrap_or_default();
            title.contains("Projectionica.Controllers")
                || title.contains("Step")
                || page_id.contains("Projectionica.Controllers")
        }),
        "repo-projected-pages endpoint should keep projected pages anchored to the external Modelica namespace"
    );
    assert_studio_json_snapshot("repo_projected_pages_endpoint_modelica_json", payload);
    Ok(())
}
