use super::{
    REPO_SEARCH_DEFAULT_LIMIT, validate_attachment_search_request, validate_autocomplete_request,
    validate_definition_request, validate_repo_search_request,
};
use crate::transport::RepoSearchRequest;

fn validate_repo_search_request_case(
    query_text: &str,
    limit: usize,
    language_filters: &[String],
    path_prefixes: &[String],
    title_filters: &[String],
    tag_filters: &[String],
    filename_filters: &[String],
) -> Result<(), String> {
    validate_repo_search_request(RepoSearchRequest {
        query_text,
        limit,
        language_filters,
        path_prefixes,
        title_filters,
        tag_filters,
        filename_filters,
    })
}

#[test]
fn repo_search_request_validation_accepts_stable_request() {
    assert!(
        validate_repo_search_request_case("rerank rust traits", 25, &[], &[], &[], &[], &[])
            .is_ok()
    );
}

#[test]
fn repo_search_request_validation_rejects_blank_query_text() {
    assert_eq!(
        validate_repo_search_request_case(
            "   ",
            REPO_SEARCH_DEFAULT_LIMIT,
            &[],
            &[],
            &[],
            &[],
            &[],
        ),
        Err("repo search query text must not be blank".to_string())
    );
}

#[test]
fn repo_search_request_validation_rejects_zero_limit() {
    assert_eq!(
        validate_repo_search_request_case("rerank rust traits", 0, &[], &[], &[], &[], &[]),
        Err("repo search limit must be greater than zero".to_string())
    );
}

#[test]
fn attachment_search_request_validation_accepts_stable_request() {
    assert!(
        validate_attachment_search_request(
            "screenshot",
            REPO_SEARCH_DEFAULT_LIMIT,
            &["png".to_string()],
            &["image".to_string()],
        )
        .is_ok()
    );
}

#[test]
fn attachment_search_request_validation_rejects_blank_extension_filters() {
    assert_eq!(
        validate_attachment_search_request(
            "screenshot",
            REPO_SEARCH_DEFAULT_LIMIT,
            &["png".to_string(), "   ".to_string()],
            &[],
        ),
        Err("attachment search extension filters must not contain blank values".to_string())
    );
}

#[test]
fn attachment_search_request_validation_rejects_blank_kind_filters() {
    assert_eq!(
        validate_attachment_search_request(
            "screenshot",
            REPO_SEARCH_DEFAULT_LIMIT,
            &[],
            &["image".to_string(), "   ".to_string()],
        ),
        Err("attachment search kind filters must not contain blank values".to_string())
    );
}

#[test]
fn definition_request_validation_accepts_stable_request() {
    assert!(validate_definition_request("AlphaService", Some("src/lib.rs"), Some(7)).is_ok());
}

#[test]
fn definition_request_validation_rejects_blank_query() {
    assert_eq!(
        validate_definition_request("   ", Some("src/lib.rs"), Some(7)),
        Err("definition query text must not be blank".to_string())
    );
}

#[test]
fn definition_request_validation_rejects_blank_source_path() {
    assert_eq!(
        validate_definition_request("AlphaService", Some("   "), Some(7)),
        Err("definition source path must not be blank".to_string())
    );
}

#[test]
fn definition_request_validation_rejects_zero_source_line() {
    assert_eq!(
        validate_definition_request("AlphaService", Some("src/lib.rs"), Some(0)),
        Err("definition source line must be greater than zero".to_string())
    );
}

#[test]
fn autocomplete_request_validation_accepts_stable_request() {
    assert!(validate_autocomplete_request("Alpha", 5).is_ok());
    assert!(validate_autocomplete_request("", 5).is_ok());
}

#[test]
fn autocomplete_request_validation_rejects_zero_limit() {
    assert_eq!(
        validate_autocomplete_request("Alpha", 0),
        Err("autocomplete limit must be greater than zero".to_string())
    );
}

#[test]
fn autocomplete_request_validation_rejects_blank_prefix() {
    assert_eq!(
        validate_autocomplete_request("   ", 5),
        Err("autocomplete prefix must not be blank".to_string())
    );
}

#[test]
fn repo_search_request_validation_rejects_blank_language_filters() {
    assert_eq!(
        validate_repo_search_request_case(
            "rerank rust traits",
            REPO_SEARCH_DEFAULT_LIMIT,
            &["rust".to_string(), "   ".to_string()],
            &[],
            &[],
            &[],
            &[],
        ),
        Err("repo search language filters must not contain blank values".to_string())
    );
}

#[test]
fn repo_search_request_validation_rejects_blank_path_prefixes() {
    assert_eq!(
        validate_repo_search_request_case(
            "rerank rust traits",
            REPO_SEARCH_DEFAULT_LIMIT,
            &[],
            &["src/".to_string(), "   ".to_string()],
            &[],
            &[],
            &[],
        ),
        Err("repo search path prefixes must not contain blank values".to_string())
    );
}

#[test]
fn repo_search_request_validation_rejects_blank_title_filters() {
    assert_eq!(
        validate_repo_search_request_case(
            "rerank rust traits",
            REPO_SEARCH_DEFAULT_LIMIT,
            &[],
            &[],
            &["README".to_string(), "   ".to_string()],
            &[],
            &[],
        ),
        Err("repo search title filters must not contain blank values".to_string())
    );
}

#[test]
fn repo_search_request_validation_rejects_blank_tag_filters() {
    assert_eq!(
        validate_repo_search_request_case(
            "rerank rust traits",
            REPO_SEARCH_DEFAULT_LIMIT,
            &[],
            &[],
            &[],
            &["lang:rust".to_string(), "   ".to_string()],
            &[],
        ),
        Err("repo search tag filters must not contain blank values".to_string())
    );
}

#[test]
fn repo_search_request_validation_rejects_blank_filename_filters() {
    assert_eq!(
        validate_repo_search_request_case(
            "rerank rust traits",
            REPO_SEARCH_DEFAULT_LIMIT,
            &[],
            &[],
            &[],
            &[],
            &["lib".to_string(), "   ".to_string()],
        ),
        Err("repo search filename filters must not contain blank values".to_string())
    );
}
