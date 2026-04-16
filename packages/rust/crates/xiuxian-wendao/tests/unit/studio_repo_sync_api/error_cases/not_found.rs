use super::super::*;

#[tokio::test]
async fn repo_projected_page_endpoint_returns_not_found_for_unknown_page() -> TestResult {
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
        "/api/repo/projected-page?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/missing.md",
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_studio_json_snapshot("repo_projected_page_not_found_error", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_tree_endpoint_returns_not_found_for_unknown_page() -> TestResult
{
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
        "/api/repo/projected-page-index-tree?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/missing.md",
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_studio_json_snapshot("repo_projected_page_index_tree_not_found_error", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_node_endpoint_returns_not_found_for_unknown_node() -> TestResult
{
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
        "/api/repo/projected-page-index-node?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&node_id=reference/solve-69592caeddee%23missing",
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_studio_json_snapshot("repo_projected_page_index_node_not_found_error", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_retrieval_hit_endpoint_returns_not_found_for_unknown_node() -> TestResult {
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
        "/api/repo/projected-retrieval-hit?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&node_id=reference/solve-69592caeddee%23missing",
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_studio_json_snapshot("repo_projected_retrieval_hit_not_found_error", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_retrieval_context_endpoint_returns_not_found_for_unknown_node() -> TestResult
{
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
        "/api/repo/projected-retrieval-context?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&node_id=reference/solve-69592caeddee%23missing",
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_studio_json_snapshot("repo_projected_retrieval_context_not_found_error", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_family_context_endpoint_returns_not_found_for_unknown_page()
-> TestResult {
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
        "/api/repo/projected-page-family-context?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/missing.md",
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_studio_json_snapshot(
        "repo_projected_page_family_context_not_found_error",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_family_cluster_endpoint_returns_not_found_for_unknown_family()
-> TestResult {
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
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "gateway-sync".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "GatewaySyncPkg.solve"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `GatewaySyncPkg.solve`"
            )
        });
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-page-family-cluster?repo=gateway-sync&page_id={}&kind=tutorial&limit=2",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_studio_json_snapshot(
        "repo_projected_page_family_cluster_not_found_error",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_navigation_endpoint_returns_not_found_for_unknown_family() -> TestResult
{
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
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "gateway-sync".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "GatewaySyncPkg.solve"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `GatewaySyncPkg.solve`"
            )
        });
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-page-navigation?repo=gateway-sync&page_id={}&family_kind=tutorial&family_limit=2",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_studio_json_snapshot("repo_projected_page_navigation_not_found_error", payload);
    Ok(())
}
