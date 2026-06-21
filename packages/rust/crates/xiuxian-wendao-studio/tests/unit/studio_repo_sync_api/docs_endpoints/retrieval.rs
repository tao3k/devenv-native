use serde_json::Value;

use crate::studio::studio_repo_sync_api_tests::support::{
    create_local_git_repo, create_local_modelica_repo, find_node_id, gateway_state_for_project,
    node_matches_needle, page_matches_needle, request_json, write_default_repo_config,
};
use crate::studio::studio_repo_sync_api_tests::{
    ProjectionPageKind, RepoProjectedPageIndexTreesQuery, RepoProjectedPagesQuery, StatusCode,
    TestResult, assert_studio_json_snapshot, fs, repo_projected_page_index_trees_from_config,
    repo_projected_pages_from_config, studio_router,
};

#[tokio::test]
async fn docs_retrieval_endpoint_returns_mixed_hits() -> TestResult {
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
        "/api/docs/retrieval?repo=gateway-sync&query=solve&kind=reference&limit=5",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_retrieval_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_retrieval_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[sources.projects.modelica-gateway-retrieval]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/retrieval?repo=modelica-gateway-retrieval&query=Projectionica.Controllers&kind=reference&limit=4",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let hits = payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("docs-retrieval payload should include a hits array")?;
    assert!(
        !hits.is_empty(),
        "docs-retrieval endpoint should return at least one mixed retrieval hit"
    );
    assert!(
        hits.len() <= 4,
        "docs-retrieval endpoint should honor the configured hit limit"
    );
    assert!(
        hits.iter().any(|hit| {
            hit.get("kind")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind == "page")
        }),
        "docs-retrieval endpoint should preserve page hits over the external Modelica path"
    );
    assert!(
        hits.iter().any(|hit| {
            hit.get("kind")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind == "page_index_node")
        }),
        "docs-retrieval endpoint should preserve page-index node hits over the external Modelica path"
    );
    assert!(
        hits.iter().all(|hit| {
            let page_anchor = hit
                .get("page")
                .and_then(Value::as_object)
                .is_some_and(|page| page_matches_needle(page, "Projectionica.Controllers"));
            let node_anchor = hit
                .get("node")
                .and_then(Value::as_object)
                .is_none_or(|node| node_matches_needle(node, "Projectionica.Controllers"));
            page_anchor && node_anchor
        }),
        "docs-retrieval endpoint hits should stay anchored to the requested Modelica controller path"
    );
    assert_studio_json_snapshot("docs_retrieval_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_retrieval_context_endpoint_returns_node_context_payload() -> TestResult {
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
        "/api/docs/retrieval-context?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&node_id=reference/solve-69592caeddee%23anchors&related_limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_retrieval_context_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_retrieval_context_endpoint_executes_over_external_modelica_plugin_path() -> TestResult
{
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[sources.projects.modelica-gateway-retrieval-context]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-retrieval-context".to_string(),
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
            repo_id: "modelica-gateway-retrieval-context".to_string(),
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
            "/api/docs/retrieval-context?repo=modelica-gateway-retrieval-context&page_id={}&node_id={}&related_limit=3",
            page.page_id, encoded_node_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("center")
            .and_then(Value::as_object)
            .and_then(|center| center.get("page"))
            .and_then(Value::as_object)
            .and_then(|page| page.get("page_id"))
            .and_then(Value::as_str)
            .is_some_and(|page_id| page_id == page.page_id),
        "docs-retrieval-context endpoint should stay anchored to the requested Modelica projected page"
    );
    assert!(
        payload
            .get("node_context")
            .and_then(Value::as_object)
            .is_some(),
        "docs-retrieval-context endpoint should include node context when reopening a Modelica page-index node"
    );
    assert_studio_json_snapshot("docs_retrieval_context_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_retrieval_hit_endpoint_returns_hit_payload() -> TestResult {
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
        "/api/docs/retrieval-hit?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_retrieval_hit_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_retrieval_hit_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[sources.projects.modelica-gateway-retrieval-hit]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-retrieval-hit".to_string(),
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
            repo_id: "modelica-gateway-retrieval-hit".to_string(),
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
            "/api/docs/retrieval-hit?repo=modelica-gateway-retrieval-hit&page_id={}&node_id={}",
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
        "docs-retrieval-hit endpoint should reopen the requested Modelica page-index node as a node hit"
    );
    assert!(
        payload
            .get("hit")
            .and_then(Value::as_object)
            .and_then(|hit| hit.get("page"))
            .and_then(Value::as_object)
            .and_then(|page_value| page_value.get("page_id"))
            .and_then(Value::as_str)
            .is_some_and(|page_id| page_id == page.page_id),
        "docs-retrieval-hit endpoint should stay anchored to the requested Modelica projected page"
    );
    assert!(
        payload
            .get("hit")
            .and_then(Value::as_object)
            .and_then(|hit| hit.get("node"))
            .and_then(Value::as_object)
            .and_then(|node| node.get("node_id"))
            .and_then(Value::as_str)
            .is_some_and(|returned_node_id| returned_node_id == node_id),
        "docs-retrieval-hit endpoint should reopen the requested Modelica page-index node"
    );
    assert_studio_json_snapshot("docs_retrieval_hit_endpoint_modelica_json", payload);
    Ok(())
}
