use crate::transport::{
    ANALYSIS_REPO_DOC_COVERAGE_ROUTE, ANALYSIS_REPO_INDEX_STATUS_ROUTE,
    ANALYSIS_REPO_OVERVIEW_ROUTE, ANALYSIS_REPO_SYNC_ROUTE, DocumentExtractMode,
    WENDAO_DOCUMENT_EXTRACT_MODE_HEADER, WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_WAIT_MS_HEADER, is_search_family_route,
    validate_document_extract_request_metadata, validate_document_extract_status_request_metadata,
    validate_markdown_analysis_request_metadata, validate_repo_doc_coverage_request_metadata,
    validate_repo_index_status_request_metadata, validate_repo_overview_request_metadata,
    validate_repo_sync_request_metadata,
};

use crate::tests::transport::server::assertions::{must_err, must_ok};
use crate::tests::transport::server::request_headers::{
    build_document_extract_metadata, build_markdown_analysis_metadata,
    build_repo_doc_coverage_metadata, build_repo_index_status_metadata,
    build_repo_overview_metadata, build_repo_sync_metadata,
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
fn validate_document_extract_request_metadata_accepts_latest_request() {
    let metadata = build_document_extract_metadata(
        "docs/manual.pdf",
        Some(".cache/document-extract"),
        Some("1"),
        Some("no"),
    );

    let request = must_ok(
        validate_document_extract_request_metadata(&metadata),
        "stable document extraction metadata should validate",
    );

    assert_eq!(request.source_path, "docs/manual.pdf");
    assert_eq!(request.output_dir, ".cache/document-extract");
    assert!(request.force);
    assert!(!request.error_row);
    assert_eq!(request.profile, "full");
    assert_eq!(request.mode, DocumentExtractMode::Sync);
    assert_eq!(request.wait_ms, 0);
}

#[test]
fn validate_document_extract_request_metadata_uses_latest_defaults() {
    let metadata = build_document_extract_metadata("docs/manual.pdf", None, None, None);

    let request = must_ok(
        validate_document_extract_request_metadata(&metadata),
        "document extraction metadata without optional headers should validate",
    );

    assert_eq!(request.output_dir, "");
    assert!(!request.force);
    assert!(request.error_row);
    assert_eq!(request.profile, "full");
    assert_eq!(request.mode, DocumentExtractMode::Sync);
    assert_eq!(request.wait_ms, 0);
}

#[test]
fn validate_document_extract_request_metadata_accepts_fast_text_profile() {
    let mut metadata = build_document_extract_metadata(
        "docs/manual.pdf",
        Some(".cache/document-extract"),
        None,
        None,
    );
    metadata.insert(
        WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER,
        tonic::metadata::MetadataValue::from_static("attachment"),
    );

    let request = must_ok(
        validate_document_extract_request_metadata(&metadata),
        "fast text document extraction profile should validate",
    );

    assert_eq!(request.profile, "fast-text");
}

#[test]
fn validate_document_extract_request_metadata_accepts_async_mode() {
    let mut metadata = build_document_extract_metadata(
        "docs/manual.pdf",
        Some(".cache/document-extract"),
        None,
        None,
    );
    metadata.insert(
        WENDAO_DOCUMENT_EXTRACT_MODE_HEADER,
        tonic::metadata::MetadataValue::from_static("async"),
    );
    metadata.insert(
        WENDAO_DOCUMENT_EXTRACT_WAIT_MS_HEADER,
        tonic::metadata::MetadataValue::from_static("250"),
    );

    let request = must_ok(
        validate_document_extract_request_metadata(&metadata),
        "async document extraction metadata should validate",
    );

    assert_eq!(request.mode, DocumentExtractMode::Async);
    assert_eq!(request.wait_ms, 250);
}

#[test]
fn validate_document_extract_request_metadata_accepts_hybrid_page_ocr_mode() {
    let mut metadata = build_document_extract_metadata(
        "docs/manual.pdf",
        Some(".cache/document-extract"),
        None,
        None,
    );
    metadata.insert(
        WENDAO_DOCUMENT_EXTRACT_MODE_HEADER,
        tonic::metadata::MetadataValue::from_static("hybrid-page-ocr"),
    );

    let request = must_ok(
        validate_document_extract_request_metadata(&metadata),
        "hybrid page OCR document extraction metadata should validate",
    );

    assert_eq!(request.mode, DocumentExtractMode::HybridPageOcr);
    assert_eq!(request.wait_ms, 0);
}

#[test]
fn validate_document_extract_status_request_metadata_requires_job_id() {
    let metadata = tonic::metadata::MetadataMap::new();

    let error = must_err(
        validate_document_extract_status_request_metadata(&metadata),
        "missing document extraction job id should fail",
    );

    assert_eq!(error.message(), "document extract job id must not be blank");
}

#[test]
fn validate_document_extract_request_metadata_rejects_invalid_bool() {
    let metadata =
        build_document_extract_metadata("docs/manual.pdf", None, Some("sometimes"), None);

    let error = must_err(
        validate_document_extract_request_metadata(&metadata),
        "invalid document extraction bool should fail",
    );

    assert_eq!(
        error.message(),
        "invalid document extract force header `x-wendao-document-extract-force`"
    );
}

#[test]
fn validate_document_extract_request_metadata_rejects_invalid_profile() {
    let mut metadata = build_document_extract_metadata("docs/manual.pdf", None, None, None);
    metadata.insert(
        WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER,
        tonic::metadata::MetadataValue::from_static("expensive-magic"),
    );

    let error = must_err(
        validate_document_extract_request_metadata(&metadata),
        "invalid document extraction profile should fail",
    );

    assert_eq!(
        error.message(),
        "unsupported document extract profile `expensive-magic`",
    );
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
