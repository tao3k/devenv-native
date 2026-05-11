use crate::transport::{
    RepoDocCoverageRequest, RepoIndexRequest, RepoProjectedPageIndexTreeRequest,
    RepoProjectedRetrievalContextInput, RepoProjectedRetrievalContextNodeId,
    RepoProjectedRetrievalContextPageId, RepoProjectedRetrievalContextRepoId,
    RepoProjectedRetrievalContextRequest, RepoSyncMode, RepoSyncRequest,
    validate_repo_doc_coverage_request, validate_repo_index_request,
    validate_repo_index_status_request, validate_repo_overview_request,
    validate_repo_projected_page_index_tree_request,
    validate_repo_projected_retrieval_context_request, validate_repo_sync_request,
};

fn validate_projected_retrieval_context_request_case(
    repo_id: &str,
    page_id: &str,
    node_id: Option<&str>,
    related_limit: Option<usize>,
) -> Result<RepoProjectedRetrievalContextRequest, String> {
    validate_repo_projected_retrieval_context_request(RepoProjectedRetrievalContextInput {
        repo_id,
        page_id,
        node_id,
        related_limit,
    })
}

#[test]
fn repo_doc_coverage_request_validation_accepts_stable_request() {
    assert_eq!(
        validate_repo_doc_coverage_request("gateway-sync", Some("GatewaySyncPkg")),
        Ok(RepoDocCoverageRequest {
            repo_id: "gateway-sync".to_string(),
            module_id: Some("GatewaySyncPkg".to_string()),
        })
    );
    assert_eq!(
        validate_repo_doc_coverage_request("gateway-sync", Some("   ")),
        Ok(RepoDocCoverageRequest {
            repo_id: "gateway-sync".to_string(),
            module_id: None,
        })
    );
}

#[test]
fn repo_overview_request_validation_accepts_stable_request() {
    assert_eq!(
        validate_repo_overview_request("gateway-sync"),
        Ok("gateway-sync".to_string())
    );
}

#[test]
fn repo_index_status_request_validation_accepts_stable_request() {
    assert_eq!(
        validate_repo_index_status_request(Some("gateway-sync")),
        Some("gateway-sync".to_string())
    );
    assert_eq!(validate_repo_index_status_request(Some("   ")), None);
    assert_eq!(validate_repo_index_status_request(None), None);
}

#[test]
fn repo_index_request_validation_accepts_stable_request() {
    assert_eq!(
        validate_repo_index_request(Some("gateway-sync"), Some("true"), "req-123"),
        Ok(RepoIndexRequest {
            repo_id: Some("gateway-sync".to_string()),
            refresh: true,
            request_id: "req-123".to_string(),
        })
    );
    assert_eq!(
        validate_repo_index_request(Some("   "), None, "req-456"),
        Ok(RepoIndexRequest {
            repo_id: None,
            refresh: false,
            request_id: "req-456".to_string(),
        })
    );
}

#[test]
fn repo_sync_request_validation_accepts_stable_request() {
    assert_eq!(
        validate_repo_sync_request("gateway-sync", Some("status")),
        Ok(RepoSyncRequest {
            repo_id: "gateway-sync".to_string(),
            mode: RepoSyncMode::Status,
        })
    );
    assert_eq!(
        validate_repo_sync_request("gateway-sync", Some("   ")),
        Ok(RepoSyncRequest {
            repo_id: "gateway-sync".to_string(),
            mode: RepoSyncMode::Ensure,
        })
    );
    assert_eq!(
        validate_repo_sync_request("gateway-sync", None),
        Ok(RepoSyncRequest {
            repo_id: "gateway-sync".to_string(),
            mode: RepoSyncMode::Ensure,
        })
    );
}

#[test]
fn repo_overview_request_validation_rejects_blank_repo() {
    assert_eq!(
        validate_repo_overview_request("   "),
        Err("repo overview repo must not be blank".to_string())
    );
}

#[test]
fn repo_doc_coverage_request_validation_rejects_blank_repo() {
    assert_eq!(
        validate_repo_doc_coverage_request("   ", Some("GatewaySyncPkg")),
        Err("repo doc coverage repo must not be blank".to_string())
    );
}

#[test]
fn repo_sync_request_validation_rejects_blank_repo() {
    assert_eq!(
        validate_repo_sync_request("   ", Some("status")),
        Err("repo sync repo must not be blank".to_string())
    );
}

#[test]
fn repo_sync_request_validation_rejects_invalid_mode() {
    assert_eq!(
        validate_repo_sync_request("gateway-sync", Some("bogus")),
        Err("unsupported repo sync mode `bogus`".to_string())
    );
}

#[test]
fn repo_index_request_validation_rejects_invalid_refresh_flag() {
    assert_eq!(
        validate_repo_index_request(Some("gateway-sync"), Some("bogus"), "req-123"),
        Err("unsupported repo index refresh flag `bogus`".to_string())
    );
}

#[test]
fn repo_index_request_validation_rejects_blank_request_id() {
    assert_eq!(
        validate_repo_index_request(Some("gateway-sync"), Some("false"), "   "),
        Err("repo index request id must not be blank".to_string())
    );
}

#[test]
fn repo_projected_page_index_tree_request_validation_accepts_stable_request() {
    assert_eq!(
        validate_repo_projected_page_index_tree_request(
            "gateway-sync",
            "repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md"
        ),
        Ok(RepoProjectedPageIndexTreeRequest {
            repo_id: "gateway-sync".to_string(),
            page_id:
                "repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md"
                    .to_string(),
        })
    );
}

#[test]
fn repo_projected_page_index_tree_request_validation_rejects_blank_repo() {
    assert_eq!(
        validate_repo_projected_page_index_tree_request("   ", "repo:gateway-sync:page"),
        Err("repo projected page-index tree repo must not be blank".to_string())
    );
}

#[test]
fn repo_projected_page_index_tree_request_validation_rejects_blank_page_id() {
    assert_eq!(
        validate_repo_projected_page_index_tree_request("gateway-sync", "   "),
        Err("repo projected page-index tree page id must not be blank".to_string())
    );
}

#[test]
fn repo_projected_retrieval_context_request_validation_accepts_stable_request() {
    assert_eq!(
        validate_projected_retrieval_context_request_case(
            "gateway-sync",
            "repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
            Some("reference/solve-69592caeddee#anchors"),
            Some(3),
        ),
        Ok(RepoProjectedRetrievalContextRequest {
            repo_id: RepoProjectedRetrievalContextRepoId::new("gateway-sync"),
            page_id: RepoProjectedRetrievalContextPageId::new(
                "repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
            ),
            node_id: Some(RepoProjectedRetrievalContextNodeId::new(
                "reference/solve-69592caeddee#anchors",
            )),
            related_limit: 3,
        })
    );
    assert_eq!(
        validate_projected_retrieval_context_request_case(
            "gateway-sync",
            "repo:gateway-sync:page",
            Some("   "),
            None,
        ),
        Ok(RepoProjectedRetrievalContextRequest {
            repo_id: RepoProjectedRetrievalContextRepoId::new("gateway-sync"),
            page_id: RepoProjectedRetrievalContextPageId::new("repo:gateway-sync:page"),
            node_id: None,
            related_limit: 5,
        })
    );
}

#[test]
fn repo_projected_retrieval_context_request_validation_rejects_invalid_request() {
    assert_eq!(
        validate_projected_retrieval_context_request_case(
            "   ",
            "repo:gateway-sync:page",
            None,
            Some(1),
        ),
        Err("repo projected retrieval-context repo must not be blank".to_string())
    );
    assert_eq!(
        validate_projected_retrieval_context_request_case("gateway-sync", "   ", None, Some(1),),
        Err("repo projected retrieval-context page id must not be blank".to_string())
    );
    assert_eq!(
        validate_projected_retrieval_context_request_case(
            "gateway-sync",
            "repo:gateway-sync:page",
            None,
            Some(0),
        ),
        Err("repo projected retrieval-context related_limit must be greater than zero".to_string())
    );
}
