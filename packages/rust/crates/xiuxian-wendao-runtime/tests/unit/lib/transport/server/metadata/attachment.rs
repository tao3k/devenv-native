use crate::transport::validate_attachment_search_request_metadata;

use super::super::assertions::{must_err, must_ok};
use super::super::request_headers::build_attachment_search_metadata;

#[test]
fn validate_attachment_search_request_metadata_accepts_stable_request() {
    let metadata = build_attachment_search_metadata(
        "image",
        "5",
        Some("png,jpg"),
        Some("image,screenshot"),
        Some("true"),
    );

    let (query_text, limit, ext_filters, kind_filters, case_sensitive) = must_ok(
        validate_attachment_search_request_metadata(&metadata),
        "stable attachment-search metadata should validate",
    );

    assert_eq!(query_text, "image");
    assert_eq!(limit, 5);
    assert!(ext_filters.contains("png"));
    assert!(ext_filters.contains("jpg"));
    assert!(kind_filters.contains("image"));
    assert!(kind_filters.contains("screenshot"));
    assert!(case_sensitive);
}

#[test]
fn validate_attachment_search_request_metadata_rejects_blank_extension_filters() {
    let metadata =
        build_attachment_search_metadata("image", "5", Some("png, "), Some("image"), None);

    let error = must_err(
        validate_attachment_search_request_metadata(&metadata),
        "blank extension filter should fail",
    );

    assert_eq!(
        error.message(),
        "attachment search extension filters must not contain blank values"
    );
}
