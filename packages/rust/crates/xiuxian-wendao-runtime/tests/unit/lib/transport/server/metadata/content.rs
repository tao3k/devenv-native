use crate::transport::{
    validate_autocomplete_request_metadata, validate_definition_request_metadata,
    validate_sql_request_metadata, validate_vfs_content_request_metadata,
    validate_vfs_resolve_request_metadata,
};

use crate::tests::transport::server::assertions::{must_err, must_ok};
use crate::tests::transport::server::request_headers::{
    build_autocomplete_metadata, build_definition_metadata, build_sql_metadata,
    build_vfs_content_metadata, build_vfs_resolve_metadata,
};

#[test]
fn validate_vfs_content_request_metadata_accepts_stable_request() {
    let metadata = build_vfs_content_metadata("main/docs/index.md");

    let path = must_ok(
        validate_vfs_content_request_metadata(&metadata),
        "stable VFS content metadata should validate",
    );

    assert_eq!(path, "main/docs/index.md");
}

#[test]
fn validate_vfs_content_request_metadata_rejects_blank_path() {
    let metadata = build_vfs_content_metadata("   ");

    let error = must_err(
        validate_vfs_content_request_metadata(&metadata),
        "blank VFS content path should fail",
    );

    assert_eq!(error.message(), "VFS content requires a non-empty path");
}

#[test]
fn validate_definition_request_metadata_accepts_stable_request() {
    let metadata = build_definition_metadata("AlphaService", Some("src/lib.rs"), Some("7"));

    let (query_text, source_path, source_line) = must_ok(
        validate_definition_request_metadata(&metadata),
        "stable definition metadata should validate",
    );

    assert_eq!(query_text, "AlphaService");
    assert_eq!(source_path.as_deref(), Some("src/lib.rs"));
    assert_eq!(source_line, Some(7));
}

#[test]
fn validate_definition_request_metadata_rejects_non_numeric_line_hint() {
    let metadata = build_definition_metadata("AlphaService", Some("src/lib.rs"), Some("abc"));

    let error = must_err(
        validate_definition_request_metadata(&metadata),
        "non-numeric definition line hint should fail",
    );

    assert_eq!(
        error.message(),
        "invalid definition line header `x-wendao-definition-line`: abc"
    );
}

#[test]
fn validate_autocomplete_request_metadata_accepts_stable_request() {
    let metadata = build_autocomplete_metadata("Alpha", "5");

    let (prefix, limit) = must_ok(
        validate_autocomplete_request_metadata(&metadata),
        "stable autocomplete metadata should validate",
    );

    assert_eq!(prefix, "Alpha");
    assert_eq!(limit, 5);
}

#[test]
fn validate_autocomplete_request_metadata_rejects_zero_limit() {
    let metadata = build_autocomplete_metadata("Alpha", "0");

    let error = must_err(
        validate_autocomplete_request_metadata(&metadata),
        "zero autocomplete limit should fail",
    );

    assert_eq!(
        error.message(),
        "autocomplete limit must be greater than zero"
    );
}

#[test]
fn validate_sql_request_metadata_accepts_read_only_query() {
    let metadata = build_sql_metadata("SELECT doc_id FROM repo_entity");

    let query_text = must_ok(
        validate_sql_request_metadata(&metadata),
        "stable SQL metadata should validate",
    );

    assert_eq!(query_text, "SELECT doc_id FROM repo_entity");
}

#[test]
fn validate_sql_request_metadata_rejects_non_query_statement() {
    let metadata = build_sql_metadata("CREATE VIEW demo AS SELECT 1");

    let error = must_err(
        validate_sql_request_metadata(&metadata),
        "non-query SQL metadata should fail",
    );

    assert_eq!(
        error.message(),
        "SQL query text must be a read-only query statement"
    );
}

#[test]
fn validate_vfs_resolve_request_metadata_accepts_stable_request() {
    let metadata = build_vfs_resolve_metadata("main/docs/index.md");

    let path = must_ok(
        validate_vfs_resolve_request_metadata(&metadata),
        "stable VFS resolve metadata should validate",
    );

    assert_eq!(path, "main/docs/index.md");
}

#[test]
fn validate_vfs_resolve_request_metadata_rejects_blank_path() {
    let metadata = build_vfs_resolve_metadata("   ");

    let error = must_err(
        validate_vfs_resolve_request_metadata(&metadata),
        "blank VFS resolve path should fail",
    );

    assert_eq!(error.message(), "VFS resolve requires a non-empty path");
}
