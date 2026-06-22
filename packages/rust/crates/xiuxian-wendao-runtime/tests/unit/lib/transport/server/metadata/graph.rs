use crate::transport::{
    ANALYSIS_MARKDOWN_ROUTE, ANALYSIS_REPO_DOC_COVERAGE_ROUTE, ANALYSIS_REPO_OVERVIEW_ROUTE,
    GRAPH_NEIGHBORS_ROUTE, SEARCH_ATTACHMENTS_ROUTE, SEARCH_INTENT_ROUTE, SEARCH_KNOWLEDGE_ROUTE,
    VFS_CONTENT_ROUTE, VFS_RESOLVE_ROUTE, is_search_family_route,
    validate_graph_neighbors_request_metadata,
};

use crate::tests::transport::server::assertions::{must_err, must_ok};
use crate::tests::transport::server::request_headers::build_graph_neighbors_metadata;

#[test]
fn validate_graph_neighbors_request_metadata_accepts_stable_request() {
    let metadata = build_graph_neighbors_metadata(
        "kernel/docs/index.md",
        Some("outgoing"),
        Some("3"),
        Some("25"),
    );

    let request = must_ok(
        validate_graph_neighbors_request_metadata(&metadata),
        "stable graph-neighbors metadata should validate",
    );

    assert_eq!(
        request,
        (
            "kernel/docs/index.md".to_string(),
            "outgoing".to_string(),
            3,
            25,
        )
    );
}

#[test]
fn validate_graph_neighbors_request_metadata_normalizes_defaults() {
    let metadata =
        build_graph_neighbors_metadata("kernel/docs/index.md", Some("invalid"), None, None);

    let request = must_ok(
        validate_graph_neighbors_request_metadata(&metadata),
        "graph-neighbors metadata should normalize defaults",
    );

    assert_eq!(
        request,
        (
            "kernel/docs/index.md".to_string(),
            "both".to_string(),
            2,
            50,
        )
    );
}

#[test]
fn validate_graph_neighbors_request_metadata_rejects_invalid_limit() {
    let metadata = build_graph_neighbors_metadata(
        "kernel/docs/index.md",
        Some("both"),
        Some("2"),
        Some("abc"),
    );

    let error = must_err(
        validate_graph_neighbors_request_metadata(&metadata),
        "non-numeric graph-neighbors limit should fail",
    );

    assert_eq!(
        error.message(),
        "invalid graph neighbors limit header `x-wendao-graph-limit`: abc"
    );
}

#[test]
fn search_family_route_matcher_accepts_semantic_business_routes() {
    assert!(is_search_family_route(SEARCH_INTENT_ROUTE));
    assert!(is_search_family_route(SEARCH_KNOWLEDGE_ROUTE));
    assert!(!is_search_family_route(SEARCH_ATTACHMENTS_ROUTE));
    assert!(!is_search_family_route(VFS_RESOLVE_ROUTE));
    assert!(!is_search_family_route(VFS_CONTENT_ROUTE));
    assert!(!is_search_family_route(GRAPH_NEIGHBORS_ROUTE));
    assert!(!is_search_family_route(ANALYSIS_MARKDOWN_ROUTE));
    assert!(!is_search_family_route(ANALYSIS_REPO_DOC_COVERAGE_ROUTE));
    assert!(!is_search_family_route(ANALYSIS_REPO_OVERVIEW_ROUTE));
}
