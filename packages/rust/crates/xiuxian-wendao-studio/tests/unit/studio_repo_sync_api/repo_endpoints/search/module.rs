use serde_json::Value;

use crate::studio::studio_repo_sync_api_tests::support::{
    create_local_git_repo, create_local_modelica_repo, gateway_state_for_project, request_json,
    write_default_repo_config,
};
use crate::studio::studio_repo_sync_api_tests::{
    StatusCode, TestResult, assert_studio_json_snapshot, fs, studio_router,
};

#[tokio::test]
async fn repo_module_search_endpoint_returns_module_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/module-search?repo=gateway-sync&query=GatewaySyncPkg&limit=5",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_module_search_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_module_search_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-module-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/module-search?repo=modelica-gateway-module-search&query=Projectionica.Controllers&limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("repo_id").and_then(Value::as_str),
        Some("modelica-gateway-module-search")
    );
    let modules = payload
        .get("modules")
        .and_then(Value::as_array)
        .ok_or("repo-module-search payload should include a modules array")?;
    assert!(
        !modules.is_empty(),
        "repo-module-search endpoint should return at least one module over the external Modelica path"
    );
    assert!(
        modules.len() <= 3,
        "repo-module-search endpoint should honor the configured module limit"
    );
    assert!(
        modules.iter().any(|module| {
            module
                .get("qualified_name")
                .and_then(Value::as_str)
                .is_some_and(|name| name.contains("Projectionica.Controllers"))
        }),
        "repo-module-search endpoint should keep module hits anchored to the requested Modelica namespace"
    );
    assert_studio_json_snapshot("repo_module_search_endpoint_modelica_json", payload);
    Ok(())
}
