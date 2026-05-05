//! Repository doc-coverage route contract and metadata validation.

use crate::transport::query_contract::{ModuleIdRef, RepoIdRef};

/// Canonical repo-doc-coverage repository metadata header for Wendao Flight requests.
pub const WENDAO_REPO_DOC_COVERAGE_REPO_HEADER: &str = "x-wendao-repo-doc-coverage-repo";
/// Canonical repo-doc-coverage module metadata header for Wendao Flight requests.
pub const WENDAO_REPO_DOC_COVERAGE_MODULE_HEADER: &str = "x-wendao-repo-doc-coverage-module";
/// Stable route for the repo doc-coverage analysis contract.
pub const ANALYSIS_REPO_DOC_COVERAGE_ROUTE: &str = "/analysis/repo-doc-coverage";

/// Validate the stable repo doc-coverage request contract.
///
/// # Errors
///
/// Returns an error when the repository identifier is blank.
pub fn validate_repo_doc_coverage_request(
    repo_id: RepoIdRef<'_>,
    module_id: Option<ModuleIdRef<'_>>,
) -> Result<(String, Option<String>), String> {
    let normalized_repo_id = repo_id.trim();
    if normalized_repo_id.is_empty() {
        return Err("repo doc coverage repo must not be blank".to_string());
    }
    let normalized_module_id = module_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    Ok((normalized_repo_id.to_string(), normalized_module_id))
}
