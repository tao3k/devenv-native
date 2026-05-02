use serde_json::Value;

use crate::gateway::studio::studio_repo_sync_api_tests::support::{
    create_local_git_repo, create_local_modelica_repo, find_node_id, gateway_state_for_project,
    request_json, write_default_repo_config, write_modelica_repo_config,
};
use crate::gateway::studio::studio_repo_sync_api_tests::{
    ProjectionPageKind, RepoProjectedPageIndexTreesQuery, RepoProjectedPagesQuery, StatusCode,
    TestResult, assert_studio_json_snapshot, fs, repo_projected_page_index_trees_from_config,
    repo_projected_pages_from_config, studio_router,
};

#[tokio::test]
async fn repo_projected_retrieval_endpoint_returns_mixed_hit_payload() -> TestResult {
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
        "/api/repo/projected-retrieval?repo=gateway-sync&query=solve&kind=reference&limit=5",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_retrieval_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_retrieval_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(
        temp.path(),
        &repo_dir,
        "modelica-gateway-projected-retrieval",
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-retrieval?repo=modelica-gateway-projected-retrieval&query=Projectionica.Controllers&kind=reference&limit=4",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let hits = payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("repo-projected-retrieval payload should include a hits array")?;
    assert!(
        !hits.is_empty(),
        "repo-projected-retrieval endpoint should return at least one mixed hit over the external Modelica path"
    );
    assert!(
        hits.len() <= 4,
        "repo-projected-retrieval endpoint should honor the configured hit limit"
    );
    assert!(
        hits.iter().any(|hit| {
            hit.get("page")
                .and_then(Value::as_object)
                .and_then(|page| page.get("title"))
                .and_then(Value::as_str)
                .is_some_and(|title| title.contains("Projectionica.Controllers"))
        }),
        "repo-projected-retrieval endpoint should keep mixed hits anchored to the requested Modelica controller path"
    );
    assert_studio_json_snapshot("repo_projected_retrieval_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_retrieval_hit_endpoint_returns_page_payload() -> TestResult {
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
        "/api/repo/projected-retrieval-hit?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_retrieval_hit_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_retrieval_hit_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-projected-retrieval-hit]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-projected-retrieval-hit".to_string(),
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
            repo_id: "modelica-gateway-projected-retrieval-hit".to_string(),
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
            "/api/repo/projected-retrieval-hit?repo=modelica-gateway-projected-retrieval-hit&page_id={}&node_id={}",
            page.page_id, encoded_node_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("hit")
            .and_then(Value::as_object)
            .and_then(|hit| hit.get("kind"))
            .and_then(Value::as_str)
            .is_some_and(|kind| kind == "page_index_node"),
        "repo-projected-retrieval-hit endpoint should reopen the requested Modelica page-index node as a node hit"
    );
    assert_studio_json_snapshot(
        "repo_projected_retrieval_hit_endpoint_modelica_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_retrieval_context_endpoint_returns_node_context_payload() -> TestResult {
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
        "/api/repo/projected-retrieval-context?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&node_id=reference/solve-69592caeddee%23anchors&related_limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_retrieval_context_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_retrieval_context_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-projected-retrieval-context]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-projected-retrieval-context".to_string(),
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
            repo_id: "modelica-gateway-projected-retrieval-context".to_string(),
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
            "/api/repo/projected-retrieval-context?repo=modelica-gateway-projected-retrieval-context&page_id={}&node_id={}&related_limit=3",
            page.page_id, encoded_node_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("node_context")
            .and_then(Value::as_object)
            .is_some(),
        "repo-projected-retrieval-context endpoint should include node context when reopening a Modelica page-index node"
    );
    assert_studio_json_snapshot(
        "repo_projected_retrieval_context_endpoint_modelica_json",
        payload,
    );
    Ok(())
}
