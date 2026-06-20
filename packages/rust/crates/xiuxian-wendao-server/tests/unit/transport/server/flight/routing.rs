use crate::transport::query_contract::{
    ANALYSIS_DOCUMENT_EXTRACT_STATUS_ROUTE, ANALYSIS_REPO_OVERVIEW_ROUTE, QUERY_SQL_ROUTE,
    REPO_SEARCH_ROUTE, SEARCH_ATTACHMENTS_ROUTE, SEARCH_AUTOCOMPLETE_ROUTE,
    SEARCH_DEFINITION_ROUTE, TOPOLOGY_3D_ROUTE, VFS_CONTENT_ROUTE,
};

use super::routing::route_payload_cacheable;

#[test]
fn route_payload_cache_policy_bypasses_mutable_search_routes() {
    for route in [
        REPO_SEARCH_ROUTE,
        SEARCH_ATTACHMENTS_ROUTE,
        SEARCH_AUTOCOMPLETE_ROUTE,
        SEARCH_DEFINITION_ROUTE,
        QUERY_SQL_ROUTE,
        ANALYSIS_DOCUMENT_EXTRACT_STATUS_ROUTE,
    ] {
        assert!(
            !route_payload_cacheable(route),
            "mutable route `{route}` must not reuse cached Flight payloads"
        );
    }
    assert!(!route_payload_cacheable("/search/knowledge"));
    assert!(!route_payload_cacheable("/search/symbols"));
    assert!(!route_payload_cacheable("/search/references"));
}

#[test]
fn route_payload_cache_policy_keeps_stable_analysis_routes_cacheable() {
    assert!(route_payload_cacheable(TOPOLOGY_3D_ROUTE));
    assert!(route_payload_cacheable(VFS_CONTENT_ROUTE));
    assert!(route_payload_cacheable(ANALYSIS_REPO_OVERVIEW_ROUTE));
}
