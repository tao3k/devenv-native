use super::super::super::*;

#[tokio::test]
async fn repo_cached_search_endpoints_return_pending_without_ready_analysis_cache() -> TestResult {
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
    write_default_repo_config_without_priming(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project_with_options(
        temp.path(),
        false,
        false,
    ));

    for uri in [
        "/api/repo/module-search?repo=gateway-sync&query=GatewaySyncPkg&limit=5",
        "/api/repo/symbol-search?repo=gateway-sync&query=solve&limit=5",
        "/api/repo/example-search?repo=gateway-sync&query=solve&limit=5",
    ] {
        let (status, payload) = request_json(router.clone(), uri).await?;
        assert_eq!(status, StatusCode::CONFLICT, "{uri}");
        assert_eq!(payload["code"], "REPO_INDEX_PENDING", "{uri}");
    }
    Ok(())
}

#[tokio::test]
async fn repo_cached_search_endpoints_can_serve_from_published_repo_entity_search_plane()
-> TestResult {
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
    let state = gateway_state_for_project_with_options(temp.path(), false, false);
    publish_repo_entity_search_plane(state.as_ref(), temp.path(), "gateway-sync").await?;
    let router = studio_router(state);

    let (module_status, module_payload) = request_json(
        router.clone(),
        "/api/repo/module-search?repo=gateway-sync&query=GatewaySyncPkg&limit=5",
    )
    .await?;
    assert_eq!(module_status, StatusCode::OK);
    assert_eq!(module_payload["repo_id"], "gateway-sync");
    assert_eq!(
        module_payload["modules"][0]["qualified_name"],
        "GatewaySyncPkg"
    );
    assert_eq!(
        module_payload["module_hits"][0]["module"]["module_id"],
        "repo:gateway-sync:module:GatewaySyncPkg"
    );

    let (symbol_status, symbol_payload) = request_json(
        router.clone(),
        "/api/repo/symbol-search?repo=gateway-sync&query=solve&limit=5",
    )
    .await?;
    assert_eq!(symbol_status, StatusCode::OK);
    assert_eq!(symbol_payload["repo_id"], "gateway-sync");
    assert_eq!(symbol_payload["symbols"][0]["name"], "solve");
    assert!(symbol_payload["symbol_hits"][0]["audit_status"].is_null());

    let (example_status, example_payload) = request_json(
        router,
        "/api/repo/example-search?repo=gateway-sync&query=solve&limit=5",
    )
    .await?;
    assert_eq!(example_status, StatusCode::OK);
    assert_eq!(example_payload["repo_id"], "gateway-sync");
    assert_eq!(example_payload["examples"][0]["title"], "solve_demo");
    assert_eq!(example_payload["example_hits"][0]["rank"], 1);
    Ok(())
}
