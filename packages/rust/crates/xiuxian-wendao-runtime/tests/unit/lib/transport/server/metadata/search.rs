use crate::transport::{validate_repo_search_request_metadata, validate_search_request_metadata};

use crate::tests::transport::server::assertions::{must_err, must_ok};
use crate::tests::transport::server::request_headers::{
    build_repo_search_metadata, build_search_metadata,
    populate_schema_and_search_headers_with_hints,
};

#[test]
fn validate_search_request_metadata_accepts_stable_request() {
    let metadata = build_search_metadata("semantic-route", "7");

    let (query_text, limit, intent, repo_hint) = must_ok(
        validate_search_request_metadata(&metadata),
        "stable search-family metadata should validate",
    );

    assert_eq!(query_text, "semantic-route");
    assert_eq!(limit, 7);
    assert_eq!(intent, None);
    assert_eq!(repo_hint, None);
}

#[test]
fn validate_search_request_metadata_accepts_intent_and_repo_hints() {
    let mut metadata = tonic::metadata::MetadataMap::new();
    populate_schema_and_search_headers_with_hints(
        &mut metadata,
        "semantic-route",
        "7",
        Some("code_search"),
        Some("gateway-sync"),
    );

    let (query_text, limit, intent, repo_hint) = must_ok(
        validate_search_request_metadata(&metadata),
        "search-family metadata with hints should validate",
    );

    assert_eq!(query_text, "semantic-route");
    assert_eq!(limit, 7);
    assert_eq!(intent.as_deref(), Some("code_search"));
    assert_eq!(repo_hint.as_deref(), Some("gateway-sync"));
}

#[test]
fn validate_search_request_metadata_rejects_blank_query_text() {
    let metadata = build_search_metadata("", "7");

    let error = must_err(
        validate_search_request_metadata(&metadata),
        "blank search-family query text should fail",
    );

    assert_eq!(error.message(), "repo search query text must not be blank");
}

#[test]
fn validate_search_request_metadata_rejects_zero_limit() {
    let metadata = build_search_metadata("semantic-route", "0");

    let error = must_err(
        validate_search_request_metadata(&metadata),
        "zero search-family limit should fail",
    );

    assert_eq!(
        error.message(),
        "repo search limit must be greater than zero"
    );
}

#[test]
fn validate_repo_search_request_metadata_accepts_repo_and_filters() {
    let metadata =
        build_repo_search_metadata("gateway-sync", "solve", "5", Some("julia"), Some("src/"));

    let request = must_ok(
        validate_repo_search_request_metadata(&metadata),
        "stable repo-search metadata should validate",
    );

    assert_eq!(request.repo_id, "gateway-sync");
    assert_eq!(request.query_text, "solve");
    assert_eq!(request.limit, 5);
    assert!(request.language_filters.contains("julia"));
    assert!(request.path_prefixes.contains("src/"));
}

#[test]
fn validate_repo_search_request_metadata_rejects_blank_repo() {
    let metadata = build_repo_search_metadata("   ", "solve", "5", None, None);

    let error: tonic::Status = must_err(
        validate_repo_search_request_metadata(&metadata),
        "blank repo-search repo should fail",
    );

    assert_eq!(
        error.message(),
        "repo search header `x-wendao-repo-search-repo` must not be blank"
    );
}
