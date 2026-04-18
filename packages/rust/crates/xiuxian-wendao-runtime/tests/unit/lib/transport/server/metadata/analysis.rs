use crate::transport::{
    ANALYSIS_REPO_DOC_COVERAGE_ROUTE, ANALYSIS_REPO_INDEX_STATUS_ROUTE,
    ANALYSIS_REPO_OVERVIEW_ROUTE, ANALYSIS_REPO_SYNC_ROUTE, is_search_family_route,
    validate_markdown_analysis_request_metadata, validate_repo_doc_coverage_request_metadata,
    validate_repo_index_status_request_metadata, validate_repo_overview_request_metadata,
    validate_repo_sync_request_metadata,
};

use super::super::assertions::{must_err, must_ok};
use super::super::request_headers::{
    build_markdown_analysis_metadata, build_repo_doc_coverage_metadata,
    build_repo_index_status_metadata, build_repo_overview_metadata, build_repo_sync_metadata,
};

#[test]
fn validate_markdown_analysis_request_metadata_accepts_stable_request() {
    let metadata = build_markdown_analysis_metadata("docs/analysis.md");

    let path = must_ok(
        validate_markdown_analysis_request_metadata(&metadata),
        "stable markdown analysis metadata should validate",
    );

    assert_eq!(path, "docs/analysis.md");
}

#[test]
fn validate_markdown_analysis_request_metadata_rejects_blank_path() {
    let metadata = build_markdown_analysis_metadata("   ");

    let error = must_err(
        validate_markdown_analysis_request_metadata(&metadata),
        "blank markdown analysis path should fail",
    );

    assert_eq!(error.message(), "markdown analysis path must not be blank");
}

#[test]
fn validate_repo_doc_coverage_request_metadata_accepts_stable_request() {
    let metadata = build_repo_doc_coverage_metadata("gateway-sync", Some("GatewaySyncPkg"));

    let (repo_id, module_id) = must_ok(
        validate_repo_doc_coverage_request_metadata(&metadata),
        "stable repo doc coverage metadata should validate",
    );

    assert_eq!(repo_id, "gateway-sync");
    assert_eq!(module_id.as_deref(), Some("GatewaySyncPkg"));
}

#[test]
fn validate_repo_doc_coverage_request_metadata_rejects_blank_repo() {
    let metadata = build_repo_doc_coverage_metadata("   ", Some("GatewaySyncPkg"));

    let error = must_err(
        validate_repo_doc_coverage_request_metadata(&metadata),
        "blank repo doc coverage repo should fail",
    );

    assert_eq!(error.message(), "repo doc coverage repo must not be blank");
}

#[test]
fn validate_repo_overview_request_metadata_accepts_stable_request() {
    let metadata = build_repo_overview_metadata("gateway-sync");

    let repo_id = must_ok(
        validate_repo_overview_request_metadata(&metadata),
        "stable repo overview metadata should validate",
    );

    assert_eq!(repo_id, "gateway-sync");
}

#[test]
fn validate_repo_overview_request_metadata_rejects_blank_repo() {
    let metadata = build_repo_overview_metadata("   ");

    let error = must_err(
        validate_repo_overview_request_metadata(&metadata),
        "blank repo overview repo should fail",
    );

    assert_eq!(error.message(), "repo overview repo must not be blank");
}

#[test]
fn validate_repo_index_status_request_metadata_accepts_stable_request() {
    let metadata = build_repo_index_status_metadata(Some("gateway-sync"));

    let repo_id = validate_repo_index_status_request_metadata(&metadata);
    assert_eq!(repo_id.as_deref(), Some("gateway-sync"));

    let metadata = build_repo_index_status_metadata(None);
    let repo_id = validate_repo_index_status_request_metadata(&metadata);
    assert_eq!(repo_id, None);
}

#[test]
fn validate_repo_sync_request_metadata_accepts_stable_request() {
    let metadata = build_repo_sync_metadata("gateway-sync", Some("status"));

    let (repo_id, mode) = must_ok(
        validate_repo_sync_request_metadata(&metadata),
        "stable repo sync metadata should validate",
    );

    assert_eq!(repo_id, "gateway-sync");
    assert_eq!(mode, "status");

    let metadata = build_repo_sync_metadata("gateway-sync", None);
    let (repo_id, mode) = must_ok(
        validate_repo_sync_request_metadata(&metadata),
        "repo sync metadata without explicit mode should validate",
    );

    assert_eq!(repo_id, "gateway-sync");
    assert_eq!(mode, "ensure");
}

#[test]
fn validate_repo_sync_request_metadata_rejects_blank_repo() {
    let metadata = build_repo_sync_metadata("   ", Some("status"));

    let error = must_err(
        validate_repo_sync_request_metadata(&metadata),
        "blank repo sync repo should fail",
    );

    assert_eq!(error.message(), "repo sync repo must not be blank");
}

#[test]
fn validate_repo_sync_request_metadata_rejects_invalid_mode() {
    let metadata = build_repo_sync_metadata("gateway-sync", Some("bogus"));

    let error = must_err(
        validate_repo_sync_request_metadata(&metadata),
        "invalid repo sync mode should fail",
    );

    assert_eq!(error.message(), "unsupported repo sync mode `bogus`");
}

#[test]
fn analysis_routes_do_not_alias_search_family_contracts() {
    assert!(!is_search_family_route(ANALYSIS_REPO_DOC_COVERAGE_ROUTE));
    assert!(!is_search_family_route(ANALYSIS_REPO_INDEX_STATUS_ROUTE));
    assert!(!is_search_family_route(ANALYSIS_REPO_OVERVIEW_ROUTE));
    assert!(!is_search_family_route(ANALYSIS_REPO_SYNC_ROUTE));
}
