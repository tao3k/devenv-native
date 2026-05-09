use super::{
    ANALYSIS_REFINE_DOC_ROUTE, ANALYSIS_REPO_DOC_COVERAGE_ROUTE, ANALYSIS_REPO_INDEX_ROUTE,
    ANALYSIS_REPO_INDEX_STATUS_ROUTE, ANALYSIS_REPO_OVERVIEW_ROUTE,
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    WENDAO_REFINE_DOC_ENTITY_ID_HEADER, WENDAO_REFINE_DOC_REPO_HEADER,
    WENDAO_REFINE_DOC_USER_HINTS_HEADER, WENDAO_REPO_DOC_COVERAGE_MODULE_HEADER,
    WENDAO_REPO_DOC_COVERAGE_REPO_HEADER, WENDAO_REPO_INDEX_REFRESH_HEADER,
    WENDAO_REPO_INDEX_REPO_HEADER, WENDAO_REPO_INDEX_REQUEST_ID_HEADER,
    WENDAO_REPO_INDEX_STATUS_REPO_HEADER, WENDAO_REPO_OVERVIEW_REPO_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER, validate_refine_doc_request,
    validate_repo_doc_coverage_request, validate_repo_index_request,
    validate_repo_index_status_request, validate_repo_overview_request,
    validate_repo_projected_page_index_tree_request,
    validate_repo_projected_retrieval_context_request, validate_repo_sync_request,
};

#[test]
fn repo_doc_coverage_route_constant_is_stable() {
    assert_eq!(
        ANALYSIS_REPO_DOC_COVERAGE_ROUTE,
        "/analysis/repo-doc-coverage"
    );
    assert_eq!(
        WENDAO_REPO_DOC_COVERAGE_REPO_HEADER,
        "x-wendao-repo-doc-coverage-repo"
    );
    assert_eq!(
        WENDAO_REPO_DOC_COVERAGE_MODULE_HEADER,
        "x-wendao-repo-doc-coverage-module"
    );
}

#[test]
fn repo_overview_route_constant_and_header_are_stable() {
    assert_eq!(ANALYSIS_REPO_OVERVIEW_ROUTE, "/analysis/repo-overview");
    assert_eq!(
        WENDAO_REPO_OVERVIEW_REPO_HEADER,
        "x-wendao-repo-overview-repo"
    );
}

#[test]
fn repo_index_status_route_constant_and_header_are_stable() {
    assert_eq!(
        ANALYSIS_REPO_INDEX_STATUS_ROUTE,
        "/analysis/repo-index-status"
    );
    assert_eq!(
        WENDAO_REPO_INDEX_STATUS_REPO_HEADER,
        "x-wendao-repo-index-status-repo"
    );
}

#[test]
fn repo_index_route_constants_and_headers_are_stable() {
    assert_eq!(ANALYSIS_REPO_INDEX_ROUTE, "/analysis/repo-index");
    assert_eq!(WENDAO_REPO_INDEX_REPO_HEADER, "x-wendao-repo-index-repo");
    assert_eq!(
        WENDAO_REPO_INDEX_REFRESH_HEADER,
        "x-wendao-repo-index-refresh"
    );
    assert_eq!(
        WENDAO_REPO_INDEX_REQUEST_ID_HEADER,
        "x-wendao-repo-index-request-id"
    );
}

#[test]
fn repo_projected_page_index_tree_route_constant_is_stable() {
    assert_eq!(
        ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
        "/analysis/repo-projected-page-index-tree"
    );
}

#[test]
fn repo_projected_retrieval_context_route_constants_are_stable() {
    assert_eq!(
        ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
        "/analysis/repo-projected-retrieval-context"
    );
    assert_eq!(
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER,
        "x-wendao-repo-projected-retrieval-context-repo"
    );
    assert_eq!(
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
        "x-wendao-repo-projected-retrieval-context-page-id"
    );
    assert_eq!(
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
        "x-wendao-repo-projected-retrieval-context-node-id"
    );
    assert_eq!(
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
        "x-wendao-repo-projected-retrieval-context-related-limit"
    );
}

#[test]
fn refine_doc_route_constant_and_headers_are_stable() {
    assert_eq!(ANALYSIS_REFINE_DOC_ROUTE, "/analysis/refine-doc");
    assert_eq!(WENDAO_REFINE_DOC_REPO_HEADER, "x-wendao-refine-doc-repo");
    assert_eq!(
        WENDAO_REFINE_DOC_ENTITY_ID_HEADER,
        "x-wendao-refine-doc-entity-id"
    );
    assert_eq!(
        WENDAO_REFINE_DOC_USER_HINTS_HEADER,
        "x-wendao-refine-doc-user-hints-b64"
    );
}

#[test]
fn repo_doc_coverage_request_validation_accepts_stable_request() {
    assert_eq!(
        validate_repo_doc_coverage_request("gateway-sync", Some("GatewaySyncPkg")),
        Ok((
            "gateway-sync".to_string(),
            Some("GatewaySyncPkg".to_string()),
        ))
    );
    assert_eq!(
        validate_repo_doc_coverage_request("gateway-sync", Some("   ")),
        Ok(("gateway-sync".to_string(), None))
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
        Ok((
            Some("gateway-sync".to_string()),
            true,
            "req-123".to_string()
        ))
    );
    assert_eq!(
        validate_repo_index_request(Some("   "), None, "req-456"),
        Ok((None, false, "req-456".to_string()))
    );
}

#[test]
fn repo_sync_request_validation_accepts_stable_request() {
    assert_eq!(
        validate_repo_sync_request("gateway-sync", Some("status")),
        Ok(("gateway-sync".to_string(), "status".to_string()))
    );
    assert_eq!(
        validate_repo_sync_request("gateway-sync", Some("   ")),
        Ok(("gateway-sync".to_string(), "ensure".to_string()))
    );
    assert_eq!(
        validate_repo_sync_request("gateway-sync", None),
        Ok(("gateway-sync".to_string(), "ensure".to_string()))
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
        Ok((
            "gateway-sync".to_string(),
            "repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md"
                .to_string(),
        ))
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
        validate_repo_projected_retrieval_context_request(
            "gateway-sync",
            "repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
            Some("reference/solve-69592caeddee#anchors"),
            Some(3),
        ),
        Ok((
            "gateway-sync".to_string(),
            "repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md"
                .to_string(),
            Some("reference/solve-69592caeddee#anchors".to_string()),
            3,
        ))
    );
    assert_eq!(
        validate_repo_projected_retrieval_context_request(
            "gateway-sync",
            "repo:gateway-sync:page",
            Some("   "),
            None,
        ),
        Ok((
            "gateway-sync".to_string(),
            "repo:gateway-sync:page".to_string(),
            None,
            5,
        ))
    );
}

#[test]
fn repo_projected_retrieval_context_request_validation_rejects_invalid_request() {
    assert_eq!(
        validate_repo_projected_retrieval_context_request(
            "   ",
            "repo:gateway-sync:page",
            None,
            Some(1),
        ),
        Err("repo projected retrieval-context repo must not be blank".to_string())
    );
    assert_eq!(
        validate_repo_projected_retrieval_context_request("gateway-sync", "   ", None, Some(1),),
        Err("repo projected retrieval-context page id must not be blank".to_string())
    );
    assert_eq!(
        validate_repo_projected_retrieval_context_request(
            "gateway-sync",
            "repo:gateway-sync:page",
            None,
            Some(0),
        ),
        Err("repo projected retrieval-context related_limit must be greater than zero".to_string())
    );
}

#[test]
fn refine_doc_request_validation_accepts_base64_user_hints() {
    assert_eq!(
        validate_refine_doc_request(
            "gateway-sync",
            "repo:gateway-sync:symbol:GatewaySyncPkg.solve",
            Some("RXhwbGFpbiB0aGlzIGVudHJ5cG9pbnQ="),
        ),
        Ok((
            "gateway-sync".to_string(),
            "repo:gateway-sync:symbol:GatewaySyncPkg.solve".to_string(),
            Some("Explain this entrypoint".to_string()),
        ))
    );
    assert_eq!(
        validate_refine_doc_request(
            "gateway-sync",
            "repo:gateway-sync:symbol:GatewaySyncPkg.solve",
            Some("   "),
        ),
        Ok((
            "gateway-sync".to_string(),
            "repo:gateway-sync:symbol:GatewaySyncPkg.solve".to_string(),
            None,
        ))
    );
}

#[test]
fn refine_doc_request_validation_rejects_blank_repo() {
    assert_eq!(
        validate_refine_doc_request("   ", "repo:gateway-sync:symbol:GatewaySyncPkg.solve", None,),
        Err("refine doc repo must not be blank".to_string())
    );
}

#[test]
fn refine_doc_request_validation_rejects_blank_entity_id() {
    assert_eq!(
        validate_refine_doc_request("gateway-sync", "   ", None),
        Err("refine doc entity_id must not be blank".to_string())
    );
}

#[test]
fn refine_doc_request_validation_rejects_invalid_base64_user_hints() {
    let Err(error) = validate_refine_doc_request(
        "gateway-sync",
        "repo:gateway-sync:symbol:GatewaySyncPkg.solve",
        Some("%%%"),
    ) else {
        panic!("invalid base64 user hints should fail");
    };
    assert!(error.starts_with("refine doc user_hints must be valid Base64:"));
}
