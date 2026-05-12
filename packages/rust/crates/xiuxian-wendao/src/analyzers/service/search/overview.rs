//! `analyzers::service::search::overview` owns Wendao service search overview behavior.

use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::{RepoOverviewQuery, RepoOverviewResult};

use crate::analyzers::service::repo_hierarchical_uri;
use crate::analyzers::service::{
    analyze_repository_from_config_with_registry, bootstrap_builtin_registry,
};

/// Build a repository overview result from normalized analysis records.
#[must_use]
pub fn build_repo_overview(
    query: &RepoOverviewQuery,
    analysis: &RepositoryAnalysisOutput,
) -> RepoOverviewResult {
    let repository = analysis.repository.as_ref();
    RepoOverviewResult {
        repo_id: query.repo_id.clone(),
        display_name: repository.map_or_else(
            || query.repo_id.clone(),
            |repository| repository.name.clone(),
        ),
        revision: repository.and_then(|repository| repository.revision.clone()),
        module_count: analysis.modules.len(),
        symbol_count: analysis.symbols.len(),
        example_count: analysis.examples.len(),
        doc_count: analysis.docs.len(),
        hierarchical_uri: Some(repo_hierarchical_uri(query.repo_id.as_str())),
        hierarchy: Some(vec!["repo".to_string(), query.repo_id.clone()]),
    }
}

/// Load configuration, analyze one repository, and return its overview.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn repo_overview_from_config_with_registry(
    query: &RepoOverviewQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepoOverviewResult, RepoIntelligenceError> {
    let analysis =
        analyze_repository_from_config_with_registry(&query.repo_id, config_path, cwd, registry)?;
    Ok(build_repo_overview(query, &analysis))
}

/// Load configuration, analyze one repository, and return its overview.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn repo_overview_from_config(
    query: &RepoOverviewQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoOverviewResult, RepoIntelligenceError> {
    let registry = bootstrap_builtin_registry()?;
    repo_overview_from_config_with_registry(query, config_path, cwd, &registry)
}
