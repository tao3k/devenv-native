//! Code-AST analysis route contract and metadata validation.

use crate::transport::query_contract::RepoIdRef;

/// Stable route for the code-AST analysis contract.
pub const ANALYSIS_CODE_AST_ROUTE: &str = "/analysis/code-ast";

/// Validate the stable code-AST analysis request contract.
///
/// # Errors
///
/// Returns an error when the repository-relative path is blank, when the repo
/// identifier is blank, or when the optional line hint is zero.
pub fn validate_code_ast_analysis_request(
    path: &str,
    repo_id: RepoIdRef<'_>,
    line_hint: Option<usize>,
) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("code AST analysis path must not be blank".to_string());
    }
    if repo_id.trim().is_empty() {
        return Err("code AST analysis repo must not be blank".to_string());
    }
    if matches!(line_hint, Some(0)) {
        return Err("code AST analysis line hint must be greater than zero".to_string());
    }
    Ok(())
}
