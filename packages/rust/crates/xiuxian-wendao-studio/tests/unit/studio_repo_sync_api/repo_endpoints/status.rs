use serde_json::Value;

use crate::studio::studio_repo_sync_api_tests::support::{
    create_local_git_repo, create_local_modelica_repo, gateway_state_for_project,
    gateway_state_for_project_with_options, publish_repo_entity_search_plane,
    redact_repo_index_payload, redact_repo_sync_payload, request_json, request_json_post,
    write_default_repo_config, write_modelica_repo_config,
};
use crate::studio::studio_repo_sync_api_tests::{
    RefineEntityDocRequest, RepoIndexRequest, StatusCode, TestResult, assert_studio_json_snapshot,
    fs, studio_router,
};

#[tokio::test]
async fn repo_index_endpoint_returns_status_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project_with_options(
        temp.path(),
        false,
        false,
    ));

    let payload = RepoIndexRequest {
        repo: Some("gateway-sync".to_string()),
        refresh: false,
    };
    let (status, mut payload) = request_json_post(router, "/api/repo/index", &payload).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(payload.get("total").and_then(Value::as_u64), Some(1));
    assert_eq!(payload.get("queued").and_then(Value::as_u64), Some(1));
    redact_repo_index_payload(&mut payload);
    assert_studio_json_snapshot("repo_index_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_index_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(temp.path(), &repo_dir, "modelica-gateway-index")?;
    let router = studio_router(gateway_state_for_project_with_options(
        temp.path(),
        false,
        false,
    ));

    let payload = RepoIndexRequest {
        repo: Some("modelica-gateway-index".to_string()),
        refresh: false,
    };
    let (status, mut payload) = request_json_post(router, "/api/repo/index", &payload).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(payload.get("total").and_then(Value::as_u64), Some(1));
    assert_eq!(payload.get("queued").and_then(Value::as_u64), Some(1));
    let repos = payload
        .get("repos")
        .and_then(Value::as_array)
        .ok_or("repo-index payload should include a repos array")?;
    assert!(
        repos.iter().any(|repo| {
            repo.get("repoId")
                .and_then(Value::as_str)
                .is_some_and(|repo_id| repo_id == "modelica-gateway-index")
        }),
        "repo-index endpoint should queue the requested external Modelica repository",
    );
    redact_repo_index_payload(&mut payload);
    assert_studio_json_snapshot("repo_index_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_index_status_endpoint_returns_status_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, mut payload) =
        request_json(router, "/api/repo/index/status?repo=gateway-sync").await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingEnabled"),
        Some(&Value::Bool(false))
    );
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingMode"),
        Some(&Value::String("deferred".to_string()))
    );
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingDeferredActivationObserved"),
        Some(&Value::Bool(true))
    );
    assert!(
        payload
            .get("studioBootstrapBackgroundIndexingDeferredActivationAt")
            .and_then(Value::as_str)
            .is_some()
    );
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingDeferredActivationSource"),
        Some(&Value::String("repo_index_status".to_string()))
    );
    assert_eq!(payload.get("total").and_then(Value::as_u64), Some(1));
    redact_repo_index_payload(&mut payload);
    assert_studio_json_snapshot("repo_index_status_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_index_status_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(temp.path(), &repo_dir, "modelica-gateway-index-status")?;
    let router = studio_router(gateway_state_for_project_with_options(
        temp.path(),
        false,
        false,
    ));

    let enqueue_payload = RepoIndexRequest {
        repo: Some("modelica-gateway-index-status".to_string()),
        refresh: false,
    };
    let (enqueue_status, _) =
        request_json_post(router.clone(), "/api/repo/index", &enqueue_payload).await?;
    assert_eq!(enqueue_status, StatusCode::OK);

    let (status, mut payload) = request_json(
        router,
        "/api/repo/index/status?repo=modelica-gateway-index-status",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingEnabled"),
        Some(&Value::Bool(false))
    );
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingMode"),
        Some(&Value::String("deferred".to_string()))
    );
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingDeferredActivationObserved"),
        Some(&Value::Bool(false))
    );
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingDeferredActivationAt"),
        Some(&Value::Null)
    );
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingDeferredActivationSource"),
        Some(&Value::Null)
    );
    assert_eq!(payload.get("total").and_then(Value::as_u64), Some(1));
    let repos = payload
        .get("repos")
        .and_then(Value::as_array)
        .ok_or("repo-index-status payload should include a repos array")?;
    assert_eq!(repos.len(), 1);
    assert_eq!(
        repos[0].get("repoId").and_then(Value::as_str),
        Some("modelica-gateway-index-status")
    );
    redact_repo_index_payload(&mut payload);
    assert_studio_json_snapshot("repo_index_status_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_refine_entity_doc_endpoint_returns_refined_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let state = gateway_state_for_project_with_options(temp.path(), false, false);
    publish_repo_entity_search_plane(state.as_ref(), temp.path(), "gateway-sync").await?;
    let router = studio_router(state);

    let payload = RefineEntityDocRequest {
        repo_id: "gateway-sync".to_string(),
        entity_id: "repo:gateway-sync:symbol:GatewaySyncPkg.solve".to_string(),
        user_hints: Some("Explain how callers should use this entrypoint.".to_string()),
    };
    let (status, payload) = request_json_post(router, "/api/analysis/refine-doc", &payload).await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_refine_entity_doc_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_sync_endpoint_returns_repo_status_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, mut payload) =
        request_json(router, "/api/repo/sync?repo=gateway-sync&mode=status").await?;
    assert_eq!(status, StatusCode::OK);
    redact_repo_sync_payload(&mut payload);
    assert_studio_json_snapshot("repo_sync_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_sync_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-sync]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, mut payload) = request_json(
        router,
        "/api/repo/sync?repo=modelica-gateway-sync&mode=status",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("repo_id").and_then(Value::as_str),
        Some("modelica-gateway-sync")
    );
    assert_eq!(payload.get("mode").and_then(Value::as_str), Some("status"));
    redact_repo_sync_payload(&mut payload);
    assert_studio_json_snapshot("repo_sync_endpoint_modelica_json", payload);
    Ok(())
}
