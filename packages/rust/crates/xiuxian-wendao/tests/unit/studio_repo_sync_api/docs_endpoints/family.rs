use super::super::*;

#[tokio::test]
async fn docs_family_context_endpoint_returns_family_context() -> TestResult {
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
        .find(|page| page.kind == ProjectionPageKind::HowTo)
        .unwrap_or_else(|| panic!("expected a projected how-to page"));
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/docs/family-context?repo=gateway-sync&page_id={}&per_kind_limit=2",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_family_context_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_family_context_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-family-context]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-family-context".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| page.kind == ProjectionPageKind::HowTo)
        .unwrap_or_else(|| panic!("expected a projected how-to page"));
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/docs/family-context?repo=modelica-gateway-family-context&page_id={}&per_kind_limit=2",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("center_page")
            .and_then(Value::as_object)
            .and_then(|center| center.get("page_id"))
            .and_then(Value::as_str)
            .is_some_and(|page_id| page_id == page.page_id),
        "docs-family-context endpoint should stay anchored to the requested Modelica how-to page"
    );
    let families = payload
        .get("families")
        .and_then(Value::as_array)
        .ok_or("docs-family-context payload should include a families array")?;
    assert!(
        !families.is_empty(),
        "docs-family-context endpoint should return at least one related family over the external Modelica path"
    );
    assert_studio_json_snapshot("docs_family_context_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_family_search_endpoint_returns_family_clusters() -> TestResult {
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
        "/api/docs/family-search?repo=gateway-sync&query=solve&kind=reference&limit=5&per_kind_limit=2",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_family_search_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_family_search_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-family-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/family-search?repo=modelica-gateway-family-search&query=Projectionica.Controllers&kind=reference&limit=3&per_kind_limit=2",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let hits = payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("docs-family-search payload should include a hits array")?;
    assert!(
        !hits.is_empty(),
        "docs-family-search endpoint should return at least one family-search hit over the external Modelica path"
    );
    assert!(
        hits.len() <= 3,
        "docs-family-search endpoint should honor the configured hit limit"
    );
    assert!(
        hits.iter().all(|hit| {
            hit.get("center_page")
                .and_then(Value::as_object)
                .is_some_and(|page| page_matches_needle(page, "Projectionica.Controllers"))
        }),
        "docs-family-search endpoint hits should stay anchored to the requested Modelica controller path"
    );
    assert_studio_json_snapshot("docs_family_search_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_family_cluster_endpoint_returns_requested_cluster() -> TestResult {
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
        .find(|page| page.kind == ProjectionPageKind::HowTo)
        .unwrap_or_else(|| panic!("expected a projected how-to page"));
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/docs/family-cluster?repo=gateway-sync&page_id={}&kind=reference&limit=2",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_family_cluster_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_family_cluster_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-family-cluster]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-family-cluster".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference && page.title == "Projectionica.Controllers"
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a module-backed projected reference page titled `Projectionica.Controllers`"
            )
        });
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/docs/family-cluster?repo=modelica-gateway-family-cluster&page_id={}&kind=how_to&limit=2",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("center_page")
            .and_then(Value::as_object)
            .and_then(|center| center.get("page_id"))
            .and_then(Value::as_str)
            .is_some_and(|page_id| page_id == page.page_id),
        "docs-family-cluster endpoint should stay anchored to the requested Modelica reference page"
    );
    assert!(
        payload
            .get("family")
            .and_then(Value::as_object)
            .and_then(|family| family.get("kind"))
            .and_then(Value::as_str)
            .is_some_and(|kind| kind == "how_to"),
        "docs-family-cluster endpoint should reopen the requested Modelica how-to family cluster"
    );
    assert_studio_json_snapshot("docs_family_cluster_endpoint_modelica_json", payload);
    Ok(())
}
