//! Markdown-analysis route contract and metadata validation.

/// Stable route for the markdown analysis contract.
pub const ANALYSIS_MARKDOWN_ROUTE: &str = "/analysis/markdown";

/// Validate the stable markdown analysis request contract.
///
/// # Errors
///
/// Returns an error when the repository-relative path is blank.
pub fn validate_markdown_analysis_request(path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("markdown analysis path must not be blank".to_string());
    }
    Ok(())
}
