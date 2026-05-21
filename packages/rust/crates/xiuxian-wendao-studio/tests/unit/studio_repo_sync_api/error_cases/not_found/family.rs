use super::support::{gateway_sync_symbol_reference_page_id, prepare_gateway_sync_repo};
use crate::studio::studio_repo_sync_api_tests::support::{gateway_state_for_project, request_json};
use crate::studio::studio_repo_sync_api_tests::{
    StatusCode, TestResult, assert_studio_json_snapshot, studio_router,
};

#[tokio::test]
async fn repo_projected_page_family_context_endpoint_returns_not_found_for_unknown_page()
-> TestResult {
    let temp = tempfile::tempdir()?;
    prepare_gateway_sync_repo(temp.path())?;
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
    prepare_gateway_sync_repo(temp.path())?;
    let page_id = gateway_sync_symbol_reference_page_id(temp.path())?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-page-family-cluster?repo=gateway-sync&page_id={page_id}&kind=tutorial&limit=2"
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
