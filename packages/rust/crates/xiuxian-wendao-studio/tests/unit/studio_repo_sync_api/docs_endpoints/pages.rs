use std::sync::Arc;

use serde_json::Value;

use crate::studio::studio_repo_sync_api_tests::support::{
    create_local_git_repo, create_local_modelica_repo, gateway_state_for_project,
    gateway_state_for_ui_config, request_json, write_default_repo_config,
};
use crate::studio::studio_repo_sync_api_tests::{
    ProjectionPageKind, RegisteredRepository, RepoProjectedPagesQuery, RepositoryPluginConfig,
    RepositoryRefreshPolicy, StatusCode, TestResult, UiConfig, UiRepoProjectConfig,
    analyze_registered_repository_with_registry, assert_studio_json_snapshot,
    build_projected_pages, fs, repo_projected_pages_from_config, studio_router,
};

#[tokio::test]
async fn docs_page_endpoint_returns_projection_payload() -> TestResult {
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
        "/api/docs/page?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_page_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_page_index_tree_endpoint_returns_tree_payload() -> TestResult {
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
        "/api/docs/page-index-tree?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_page_index_tree_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_page_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-page]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-page".to_string(),
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
            "/api/docs/page?repo=modelica-gateway-page&page_id={}",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("page")
            .and_then(Value::as_object)
            .and_then(|page_value| page_value.get("page_id"))
            .and_then(Value::as_str)
            .is_some_and(|page_id| page_id == page.page_id),
        "docs-page endpoint should stay anchored to the requested Modelica projected page"
    );
    assert!(
        payload
            .get("page")
            .and_then(Value::as_object)
            .and_then(|page_value| page_value.get("title"))
            .and_then(Value::as_str)
            .is_some_and(|title| title == "Projectionica.Controllers.PI"),
        "docs-page endpoint should reopen the requested Modelica projected page title"
    );
    assert_studio_json_snapshot("docs_page_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_page_endpoint_uses_studio_state_without_persisted_config_file() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;

    let plugin_registry = Arc::new(
        xiuxian_wendao::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap builtin plugin registry: {error}")),
    );
    let repository = RegisteredRepository {
        id: "gateway-sync".to_string(),
        path: Some(repo_dir.clone()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
    };
    let analysis =
        analyze_registered_repository_with_registry(&repository, temp.path(), &plugin_registry)?;
    let page_id = build_projected_pages(&analysis)
        .into_iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "Projectionica.Controllers.PI"
                && page.page_id.contains(":symbol:")
        })
        .map(|page| page.page_id)
        .ok_or("expected a projected symbol-backed page for `Projectionica.Controllers.PI`")?;

    let router = studio_router(gateway_state_for_ui_config(
        temp.path(),
        UiConfig {
            projects: Vec::new(),
            repo_projects: vec![UiRepoProjectConfig {
                id: "gateway-sync".to_string(),
                root: Some(repo_dir.display().to_string()),
                url: None,
                git_ref: None,
                refresh: None,
                plugins: vec!["modelica".to_string()],
            }],
        },
        plugin_registry,
    ));

    let (status, payload) = request_json(
        router,
        &format!("/api/docs/page?repo=gateway-sync&page_id={page_id}"),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload["page"]["title"].as_str(),
        Some("Projectionica.Controllers.PI")
    );
    Ok(())
}
