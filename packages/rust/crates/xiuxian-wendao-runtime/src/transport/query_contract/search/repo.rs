//! Repository search route contract and metadata validation.

/// Canonical repo-search query text metadata header for Wendao Flight requests.
pub const WENDAO_REPO_SEARCH_QUERY_HEADER: &str = "x-wendao-repo-search-query";
/// Canonical repo-search result-limit metadata header for Wendao Flight requests.
pub const WENDAO_REPO_SEARCH_LIMIT_HEADER: &str = "x-wendao-repo-search-limit";
/// Canonical repo-search repository metadata header for Wendao Flight requests.
pub const WENDAO_REPO_SEARCH_REPO_HEADER: &str = "x-wendao-repo-search-repo";
/// Canonical repo-search language-filter metadata header for Wendao Flight requests.
pub const WENDAO_REPO_SEARCH_LANGUAGE_FILTERS_HEADER: &str =
    "x-wendao-repo-search-language-filters";
/// Canonical repo-search path-prefix metadata header for Wendao Flight requests.
pub const WENDAO_REPO_SEARCH_PATH_PREFIXES_HEADER: &str = "x-wendao-repo-search-path-prefixes";
/// Canonical repo-search title-filter metadata header for Wendao Flight requests.
pub const WENDAO_REPO_SEARCH_TITLE_FILTERS_HEADER: &str = "x-wendao-repo-search-title-filters";
/// Canonical repo-search tag-filter metadata header for Wendao Flight requests.
pub const WENDAO_REPO_SEARCH_TAG_FILTERS_HEADER: &str = "x-wendao-repo-search-tag-filters";
/// Canonical repo-search filename-filter metadata header for Wendao Flight requests.
pub const WENDAO_REPO_SEARCH_FILENAME_FILTERS_HEADER: &str =
    "x-wendao-repo-search-filename-filters";
/// Stable route for the repo-search query contract.
pub const REPO_SEARCH_ROUTE: &str = "/search/repos/main";
/// Stable default result limit for repo-search requests.
pub const REPO_SEARCH_DEFAULT_LIMIT: usize = 10;
/// Canonical repo-search response `doc_id` column.
pub const REPO_SEARCH_DOC_ID_COLUMN: &str = "doc_id";
/// Canonical repo-search response `path` column.
pub const REPO_SEARCH_PATH_COLUMN: &str = "path";
/// Canonical repo-search response `title` column.
pub const REPO_SEARCH_TITLE_COLUMN: &str = "title";
/// Canonical repo-search response `best_section` column.
pub const REPO_SEARCH_BEST_SECTION_COLUMN: &str = "best_section";
/// Canonical repo-search response `match_reason` column.
pub const REPO_SEARCH_MATCH_REASON_COLUMN: &str = "match_reason";
/// Canonical repo-search response navigation-path column.
pub const REPO_SEARCH_NAVIGATION_PATH_COLUMN: &str = "navigation_path";
/// Canonical repo-search response navigation-category column.
pub const REPO_SEARCH_NAVIGATION_CATEGORY_COLUMN: &str = "navigation_category";
/// Canonical repo-search response navigation-line column.
pub const REPO_SEARCH_NAVIGATION_LINE_COLUMN: &str = "navigation_line";
/// Canonical repo-search response navigation-line-end column.
pub const REPO_SEARCH_NAVIGATION_LINE_END_COLUMN: &str = "navigation_line_end";
/// Canonical repo-search response hierarchy column.
pub const REPO_SEARCH_HIERARCHY_COLUMN: &str = "hierarchy";
/// Canonical repo-search response `tags` column.
pub const REPO_SEARCH_TAGS_COLUMN: &str = "tags";
/// Canonical repo-search response `score` column.
pub const REPO_SEARCH_SCORE_COLUMN: &str = "score";
/// Canonical repo-search response `language` column.
pub const REPO_SEARCH_LANGUAGE_COLUMN: &str = "language";

/// Validate the stable repo-search request contract.
///
/// # Errors
///
/// Returns an error when the repo-search query text is blank or the requested
/// limit is zero.
pub fn validate_repo_search_request(
    query_text: &str,
    limit: usize,
    language_filters: &[String],
    path_prefixes: &[String],
    title_filters: &[String],
    tag_filters: &[String],
    filename_filters: &[String],
) -> Result<(), String> {
    if query_text.trim().is_empty() {
        return Err("repo search query text must not be blank".to_string());
    }
    if limit == 0 {
        return Err("repo search limit must be greater than zero".to_string());
    }
    for language_filter in language_filters {
        if language_filter.trim().is_empty() {
            return Err("repo search language filters must not contain blank values".to_string());
        }
    }
    for path_prefix in path_prefixes {
        if path_prefix.trim().is_empty() {
            return Err("repo search path prefixes must not contain blank values".to_string());
        }
    }
    for title_filter in title_filters {
        if title_filter.trim().is_empty() {
            return Err("repo search title filters must not contain blank values".to_string());
        }
    }
    for tag_filter in tag_filters {
        if tag_filter.trim().is_empty() {
            return Err("repo search tag filters must not contain blank values".to_string());
        }
    }
    for filename_filter in filename_filters {
        if filename_filter.trim().is_empty() {
            return Err("repo search filename filters must not contain blank values".to_string());
        }
    }
    Ok(())
}
