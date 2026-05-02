use crate::gateway::studio::studio_repo_sync_api_tests::support::{
    gateway_state_for_project, request_json,
};
use crate::gateway::studio::studio_repo_sync_api_tests::{
    StatusCode, TestResult, assert_studio_json_snapshot, studio_router,
};

#[tokio::test]
async fn repo_gateway_returns_missing_repo_error() -> TestResult {
    let temp = tempfile::tempdir()?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    for uri in [
        "/api/docs/page-index-tree?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
        "/api/repo/overview",
        "/api/repo/module-search?query=solve",
        "/api/repo/symbol-search?query=solve",
        "/api/repo/example-search?query=solve",
        "/api/repo/projected-page?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
        "/api/repo/projected-page-index-node?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&node_id=reference/solve-69592caeddee%23anchors",
        "/api/repo/projected-retrieval-hit?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
        "/api/repo/projected-retrieval-context?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
        "/api/repo/projected-page-family-context?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
        "/api/repo/projected-page-family-search?query=solve",
        "/api/repo/projected-page-family-cluster?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&kind=reference",
        "/api/repo/projected-page-navigation?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
        "/api/repo/projected-page-navigation-search?query=solve",
        "/api/repo/projected-page-index-tree?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
        "/api/repo/projected-page-index-tree-search?query=anchors",
        "/api/repo/projected-page-search?query=solve",
        "/api/repo/projected-retrieval?query=solve",
        "/api/repo/doc-coverage",
        "/api/repo/sync",
        "/api/repo/projected-pages",
        "/api/repo/projected-page-index-trees",
    ] {
        let (status, payload) = request_json(router.clone(), uri).await?;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
        assert_studio_json_snapshot("repo_gateway_missing_repo_error", payload);
    }
    Ok(())
}

#[tokio::test]
async fn repo_gateway_search_endpoints_require_query_param() -> TestResult {
    let temp = tempfile::tempdir()?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    for uri in [
        "/api/repo/module-search?repo=gateway-sync",
        "/api/repo/symbol-search?repo=gateway-sync",
        "/api/repo/example-search?repo=gateway-sync",
        "/api/repo/projected-page-index-tree-search?repo=gateway-sync",
        "/api/repo/projected-page-search?repo=gateway-sync",
        "/api/repo/projected-page-family-search?repo=gateway-sync",
        "/api/repo/projected-page-navigation-search?repo=gateway-sync",
        "/api/repo/projected-retrieval?repo=gateway-sync",
    ] {
        let (status, payload) = request_json(router.clone(), uri).await?;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
        assert_studio_json_snapshot("repo_gateway_missing_query_error", payload);
    }
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_endpoint_requires_page_id() -> TestResult {
    let temp = tempfile::tempdir()?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    for uri in [
        "/api/docs/page-index-tree?repo=gateway-sync",
        "/api/repo/projected-page?repo=gateway-sync",
        "/api/repo/projected-page-index-node?repo=gateway-sync&node_id=reference/solve-69592caeddee%23anchors",
        "/api/repo/projected-retrieval-hit?repo=gateway-sync",
        "/api/repo/projected-retrieval-context?repo=gateway-sync",
        "/api/repo/projected-page-family-context?repo=gateway-sync",
        "/api/repo/projected-page-family-cluster?repo=gateway-sync&kind=reference",
        "/api/repo/projected-page-navigation?repo=gateway-sync",
        "/api/repo/projected-page-index-tree?repo=gateway-sync",
    ] {
        let (status, payload) = request_json(router.clone(), uri).await?;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
        assert_studio_json_snapshot("repo_gateway_missing_page_id_error", payload);
    }
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_node_endpoint_requires_node_id() -> TestResult {
    let temp = tempfile::tempdir()?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-index-node?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
    )
    .await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_studio_json_snapshot("repo_gateway_missing_node_id_error", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_family_cluster_endpoint_requires_kind() -> TestResult {
    let temp = tempfile::tempdir()?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-family-cluster?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
    )
    .await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_studio_json_snapshot("repo_gateway_missing_kind_error", payload);
    Ok(())
}

#[tokio::test]
async fn repo_sync_endpoint_rejects_invalid_mode() -> TestResult {
    let temp = tempfile::tempdir()?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) =
        request_json(router, "/api/repo/sync?repo=gateway-sync&mode=bogus").await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_studio_json_snapshot("repo_sync_endpoint_invalid_mode_error", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_search_endpoint_rejects_invalid_kind() -> TestResult {
    let temp = tempfile::tempdir()?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    for uri in [
        "/api/repo/projected-page-search?repo=gateway-sync&query=solve&kind=bogus",
        "/api/repo/projected-page-family-cluster?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&kind=bogus",
        "/api/repo/projected-page-family-search?repo=gateway-sync&query=solve&kind=bogus",
        "/api/repo/projected-page-navigation?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&family_kind=bogus",
        "/api/repo/projected-page-navigation-search?repo=gateway-sync&query=solve&family_kind=bogus",
        "/api/repo/projected-page-navigation-search?repo=gateway-sync&query=solve&kind=bogus",
        "/api/repo/projected-page-index-tree-search?repo=gateway-sync&query=anchors&kind=bogus",
        "/api/repo/projected-retrieval?repo=gateway-sync&query=solve&kind=bogus",
    ] {
        let (status, payload) = request_json(router.clone(), uri).await?;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
        assert_studio_json_snapshot("repo_projected_page_search_invalid_kind_error", payload);
    }
    Ok(())
}
