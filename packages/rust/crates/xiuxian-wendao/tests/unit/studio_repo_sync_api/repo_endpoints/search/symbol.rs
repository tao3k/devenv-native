use super::super::super::*;

#[tokio::test]
async fn repo_symbol_search_endpoint_returns_symbol_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\nsolve() = nothing\nend\n",
    )?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/symbol-search?repo=gateway-sync&query=solve&limit=5",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_symbol_search_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_symbol_search_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-symbol-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/symbol-search?repo=modelica-gateway-symbol-search&query=PI&limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("repo_id").and_then(Value::as_str),
        Some("modelica-gateway-symbol-search")
    );
    let symbols = payload
        .get("symbols")
        .and_then(Value::as_array)
        .ok_or("repo-symbol-search payload should include a symbols array")?;
    assert!(
        !symbols.is_empty(),
        "repo-symbol-search endpoint should return at least one symbol over the external Modelica path"
    );
    assert!(
        symbols.len() <= 3,
        "repo-symbol-search endpoint should honor the configured symbol limit"
    );
    assert!(
        symbols.iter().any(|symbol| {
            symbol
                .get("qualified_name")
                .and_then(Value::as_str)
                .is_some_and(|name| name.contains("Projectionica.Controllers.PI"))
        }),
        "repo-symbol-search endpoint should keep symbol hits anchored to the requested Modelica symbol"
    );
    assert_studio_json_snapshot("repo_symbol_search_endpoint_modelica_json", payload);
    Ok(())
}
