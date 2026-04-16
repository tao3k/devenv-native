use super::super::*;

#[tokio::test]
async fn repo_projected_page_index_tree_endpoint_returns_tree_payload() -> TestResult {
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
        "/api/repo/projected-page-index-tree?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_page_index_tree_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_tree_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-projected-index-tree]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-projected-index-tree".to_string(),
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
            "/api/repo/projected-page-index-tree?repo=modelica-gateway-projected-index-tree&page_id={}",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("tree")
            .and_then(Value::as_object)
            .and_then(|tree| tree.get("page_id"))
            .and_then(Value::as_str)
            .is_some_and(|page_id| page_id == page.page_id),
        "repo-projected-page-index-tree endpoint should stay anchored to the requested Modelica projected page"
    );
    assert!(
        payload
            .get("tree")
            .and_then(Value::as_object)
            .and_then(|tree| tree.get("roots"))
            .and_then(Value::as_array)
            .is_some_and(|roots| !roots.is_empty()),
        "repo-projected-page-index-tree endpoint should reopen a non-empty tree over the external Modelica path"
    );
    assert_studio_json_snapshot(
        "repo_projected_page_index_tree_endpoint_modelica_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_node_endpoint_returns_node_payload() -> TestResult {
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
        "/api/repo/projected-page-index-node?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&node_id=reference/solve-69592caeddee%23anchors",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_page_index_node_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_node_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-projected-index-node]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-projected-index-node".to_string(),
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
    let trees = repo_projected_page_index_trees_from_config(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: "modelica-gateway-projected-index-node".to_string(),
        },
        None,
        temp.path(),
    )?;
    let tree = trees
        .trees
        .iter()
        .find(|tree| tree.page_id == page.page_id)
        .unwrap_or_else(|| panic!("expected a projected page-index tree for the selected page"));
    let node_id = find_node_id(tree.roots.as_slice(), "Anchors")
        .unwrap_or_else(|| panic!("expected a projected page-index node titled `Anchors`"));
    let encoded_node_id = node_id.replace('#', "%23");
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-page-index-node?repo=modelica-gateway-projected-index-node&page_id={}&node_id={}",
            page.page_id, encoded_node_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("hit")
            .and_then(Value::as_object)
            .and_then(|hit| hit.get("node_id"))
            .and_then(Value::as_str)
            .is_some_and(|returned_node_id| returned_node_id == node_id),
        "repo-projected-page-index-node endpoint should reopen the requested Modelica page-index node"
    );
    assert_studio_json_snapshot(
        "repo_projected_page_index_node_endpoint_modelica_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_tree_search_endpoint_returns_hit_payload() -> TestResult {
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
        "/api/repo/projected-page-index-tree-search?repo=gateway-sync&query=anchors&kind=reference&limit=5",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot(
        "repo_projected_page_index_tree_search_endpoint_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_tree_search_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(
        temp.path(),
        &repo_dir,
        "modelica-gateway-projected-index-tree-search",
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-index-tree-search?repo=modelica-gateway-projected-index-tree-search&query=anchors&kind=reference&limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let hits = payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("repo-projected-page-index-tree-search payload should include a hits array")?;
    assert!(
        !hits.is_empty(),
        "repo-projected-page-index-tree-search endpoint should return at least one section hit over the external Modelica path"
    );
    assert!(
        hits.len() <= 3,
        "repo-projected-page-index-tree-search endpoint should honor the configured hit limit"
    );
    assert!(
        hits.iter().any(|hit| {
            hit.get("page_title")
                .and_then(Value::as_str)
                .is_some_and(|title| title.contains("Projectionica.Controllers"))
        }),
        "repo-projected-page-index-tree-search endpoint should keep section hits anchored to the requested Modelica controller family"
    );
    assert_studio_json_snapshot(
        "repo_projected_page_index_tree_search_endpoint_modelica_json",
        payload,
    );
    Ok(())
}
