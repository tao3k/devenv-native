use serde_json::Value;

use crate::gateway::studio::studio_repo_sync_api_tests::support::{
    create_local_git_repo, create_local_modelica_repo, find_node_id, gateway_state_for_project,
    projected_page_and_node_id_for_title, request_json, write_default_repo_config,
    write_modelica_repo_config,
};
use crate::gateway::studio::studio_repo_sync_api_tests::{
    ProjectionPageKind, RepoProjectedPageIndexTreesQuery, RepoProjectedPagesQuery, StatusCode,
    TestResult, assert_studio_json_snapshot, fs, repo_projected_page_index_trees_from_config,
    repo_projected_pages_from_config, studio_router,
};

#[tokio::test]
async fn repo_projected_page_navigation_endpoint_returns_navigation_bundle() -> TestResult {
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
    let trees = repo_projected_page_index_trees_from_config(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: "gateway-sync".to_string(),
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
            "/api/repo/projected-page-navigation?repo=gateway-sync&page_id={}&node_id={}&family_kind=how_to&related_limit=3&family_limit=2",
            page.page_id, encoded_node_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_page_navigation_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_navigation_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(
        temp.path(),
        &repo_dir,
        "modelica-gateway-projected-navigation",
    )?;
    let (page_id, node_id) = projected_page_and_node_id_for_title(
        temp.path(),
        "modelica-gateway-projected-navigation",
        "Projectionica.Controllers.PI",
        "Anchors",
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-page-navigation?repo=modelica-gateway-projected-navigation&page_id={}&node_id={}&family_kind=how_to&related_limit=3&family_limit=2",
            page_id,
            node_id.replace('#', "%23")
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload
            .get("center")
            .and_then(Value::as_object)
            .and_then(|center| center.get("page"))
            .and_then(Value::as_object)
            .and_then(|page| page.get("page_id"))
            .and_then(Value::as_str),
        Some(page_id.as_str())
    );
    assert_studio_json_snapshot(
        "repo_projected_page_navigation_endpoint_modelica_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_navigation_search_endpoint_returns_navigation_hits() -> TestResult {
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
        "/api/repo/projected-page-navigation-search?repo=gateway-sync&query=solve&kind=reference&family_kind=how_to&limit=5&related_limit=3&family_limit=2",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot(
        "repo_projected_page_navigation_search_endpoint_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_navigation_search_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(
        temp.path(),
        &repo_dir,
        "modelica-gateway-projected-navigation-search",
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-navigation-search?repo=modelica-gateway-projected-navigation-search&query=Projectionica.Controllers&kind=reference&family_kind=how_to&limit=2&related_limit=3&family_limit=2",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let hits = payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("repo-projected-page-navigation-search payload should include a hits array")?;
    assert!(
        !hits.is_empty(),
        "repo-projected-page-navigation-search endpoint should return at least one navigation hit over the external Modelica path"
    );
    assert!(
        hits.len() <= 2,
        "repo-projected-page-navigation-search endpoint should honor the configured hit limit"
    );
    assert!(
        hits.iter().any(|hit| {
            hit.get("navigation")
                .and_then(Value::as_object)
                .and_then(|navigation| navigation.get("center"))
                .and_then(Value::as_object)
                .and_then(|center| center.get("page"))
                .and_then(Value::as_object)
                .and_then(|page| page.get("title"))
                .and_then(Value::as_str)
                .is_some_and(|title| title.contains("Projectionica.Controllers"))
        }),
        "repo-projected-page-navigation-search endpoint should keep navigation hits anchored to the requested Modelica controller path"
    );
    assert_studio_json_snapshot(
        "repo_projected_page_navigation_search_endpoint_modelica_json",
        payload,
    );
    Ok(())
}
