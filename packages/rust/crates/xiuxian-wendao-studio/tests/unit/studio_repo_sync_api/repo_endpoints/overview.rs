use serde_json::Value;

use crate::studio::studio_repo_sync_api_tests::support::{
    create_local_git_repo, create_local_modelica_repo, gateway_state_for_project_with_options,
    publish_repo_entity_search_plane, redact_repo_overview_payload, request_json,
    write_default_repo_config, write_default_repo_config_without_priming,
};
use crate::studio::studio_repo_sync_api_tests::{
    StatusCode, TestResult, assert_studio_json_snapshot, fs, studio_router,
};

#[tokio::test]
async fn repo_overview_endpoint_returns_repo_summary_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let state = gateway_state_for_project_with_options(temp.path(), false, false);
    publish_repo_entity_search_plane(state.as_ref(), temp.path(), "gateway-sync").await?;
    let router = studio_router(state);

    let (status, mut payload) =
        request_json(router, "/api/repo/overview?repo=gateway-sync").await?;
    assert_eq!(status, StatusCode::OK);
    redact_repo_overview_payload(&mut payload);
    assert_studio_json_snapshot("repo_overview_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_overview_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-overview]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let state = gateway_state_for_project_with_options(temp.path(), false, false);
    publish_repo_entity_search_plane(state.as_ref(), temp.path(), "modelica-gateway-overview")
        .await?;
    let router = studio_router(state);

    let (status, mut payload) =
        request_json(router, "/api/repo/overview?repo=modelica-gateway-overview").await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("repo_id").and_then(Value::as_str),
        Some("modelica-gateway-overview")
    );
    assert_eq!(
        payload.get("display_name").and_then(Value::as_str),
        Some("Projectionica")
    );
    assert!(
        payload
            .get("module_count")
            .and_then(Value::as_u64)
            .is_some_and(|count| count >= 1),
        "repo-overview endpoint should expose at least one Modelica module over the external plugin path"
    );
    redact_repo_overview_payload(&mut payload);
    assert_studio_json_snapshot("repo_overview_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_overview_endpoint_returns_index_not_ready_without_published_repo_entity() -> TestResult
{
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    write_default_repo_config_without_priming(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project_with_options(
        temp.path(),
        false,
        false,
    ));

    let (status, payload) = request_json(router, "/api/repo/overview?repo=gateway-sync").await?;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(payload["code"], "INDEX_NOT_READY");
    assert_eq!(payload["details"], "repo_entity");
    Ok(())
}
