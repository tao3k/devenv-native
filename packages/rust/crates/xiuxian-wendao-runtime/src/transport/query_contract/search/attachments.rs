//! Attachment search route contract and metadata validation.

use super::repo::{RepoSearchRequest, validate_repo_search_request};

/// Canonical attachment-search extension-filter metadata header for Wendao Flight requests.
pub const WENDAO_ATTACHMENT_SEARCH_EXT_FILTERS_HEADER: &str =
    "x-wendao-attachment-search-ext-filters";
/// Canonical attachment-search kind-filter metadata header for Wendao Flight requests.
pub const WENDAO_ATTACHMENT_SEARCH_KIND_FILTERS_HEADER: &str =
    "x-wendao-attachment-search-kind-filters";
/// Canonical attachment-search case-sensitive metadata header for Wendao Flight requests.
pub const WENDAO_ATTACHMENT_SEARCH_CASE_SENSITIVE_HEADER: &str =
    "x-wendao-attachment-search-case-sensitive";
/// Stable route for the search-attachments contract.
pub const SEARCH_ATTACHMENTS_ROUTE: &str = "/search/attachments";

/// Validate the stable attachment-search request contract.
///
/// # Errors
///
/// Returns an error when the attachment-search query text is blank, the
/// requested limit is zero, or any declared extension/kind filter is blank.
pub fn validate_attachment_search_request(
    query_text: &str,
    limit: usize,
    ext_filters: &[String],
    kind_filters: &[String],
) -> Result<(), String> {
    validate_repo_search_request(RepoSearchRequest {
        query_text,
        limit,
        language_filters: &[],
        path_prefixes: &[],
        title_filters: &[],
        tag_filters: &[],
        filename_filters: &[],
    })?;
    for ext_filter in ext_filters {
        if ext_filter.trim().is_empty() {
            return Err(
                "attachment search extension filters must not contain blank values".to_string(),
            );
        }
    }
    for kind_filter in kind_filters {
        if kind_filter.trim().is_empty() {
            return Err("attachment search kind filters must not contain blank values".to_string());
        }
    }
    Ok(())
}
