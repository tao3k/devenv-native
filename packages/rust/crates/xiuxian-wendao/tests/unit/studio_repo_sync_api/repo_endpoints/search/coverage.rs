use super::super::super::*;

#[tokio::test]
async fn repo_doc_coverage_endpoint_returns_coverage_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    fs::write(repo_dir.join("docs").join("Problem.md"), "# Problem\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/doc-coverage?repo=gateway-sync&module=GatewaySyncPkg",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_doc_coverage_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_doc_coverage_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-doc-coverage]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/doc-coverage?repo=modelica-gateway-doc-coverage&module=Projectionica.Controllers",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("repo_id").and_then(Value::as_str),
        Some("modelica-gateway-doc-coverage")
    );
    assert!(
        payload
            .get("module_id")
            .and_then(Value::as_str)
            .is_some_and(|module_id| module_id.contains("Projectionica.Controllers")),
        "repo-doc-coverage endpoint should stay anchored to the requested Modelica module"
    );
    let docs = payload
        .get("docs")
        .and_then(Value::as_array)
        .ok_or("repo-doc-coverage payload should include a docs array")?;
    assert!(
        !docs.is_empty(),
        "repo-doc-coverage endpoint should expose at least one documentation record over the external Modelica path"
    );
    assert_studio_json_snapshot("repo_doc_coverage_endpoint_modelica_json", payload);
    Ok(())
}
