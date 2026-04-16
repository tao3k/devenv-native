use super::super::super::*;

#[tokio::test]
async fn repo_example_search_endpoint_returns_example_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/example-search?repo=gateway-sync&query=solve&limit=5",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_example_search_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_example_search_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-example-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/example-search?repo=modelica-gateway-example-search&query=Step&limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("repo_id").and_then(Value::as_str),
        Some("modelica-gateway-example-search")
    );
    let examples = payload
        .get("examples")
        .and_then(Value::as_array)
        .ok_or("repo-example-search payload should include an examples array")?;
    assert!(
        !examples.is_empty(),
        "repo-example-search endpoint should return at least one example over the external Modelica path"
    );
    assert!(
        examples.len() <= 3,
        "repo-example-search endpoint should honor the configured example limit"
    );
    assert!(
        examples.iter().any(|example| {
            let title = example
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let path = example
                .get("path")
                .and_then(Value::as_str)
                .unwrap_or_default();
            title.contains("Step") || path.contains("Step.mo")
        }),
        "repo-example-search endpoint should keep example hits anchored to the requested Modelica example"
    );
    assert_studio_json_snapshot("repo_example_search_endpoint_modelica_json", payload);
    Ok(())
}
