//! Definition search route contract and metadata validation.

/// Canonical definition-resolution query metadata header for Wendao Flight requests.
pub const WENDAO_DEFINITION_QUERY_HEADER: &str = "x-wendao-definition-query";
/// Canonical definition-resolution source-path metadata header for Wendao Flight requests.
pub const WENDAO_DEFINITION_PATH_HEADER: &str = "x-wendao-definition-path";
/// Canonical definition-resolution source-line metadata header for Wendao Flight requests.
pub const WENDAO_DEFINITION_LINE_HEADER: &str = "x-wendao-definition-line";
/// Stable route for the definition-resolution contract.
pub const SEARCH_DEFINITION_ROUTE: &str = "/search/definition";

/// Validate the stable definition-resolution request contract.
///
/// # Errors
///
/// Returns an error when the definition query text is blank, when the optional
/// source path is blank, or when the optional source line is zero.
pub fn validate_definition_request(
    query_text: &str,
    source_path: Option<&str>,
    source_line: Option<usize>,
) -> Result<(), String> {
    if query_text.trim().is_empty() {
        return Err("definition query text must not be blank".to_string());
    }
    if matches!(source_path, Some(path) if path.trim().is_empty()) {
        return Err("definition source path must not be blank".to_string());
    }
    if matches!(source_line, Some(0)) {
        return Err("definition source line must be greater than zero".to_string());
    }
    Ok(())
}
