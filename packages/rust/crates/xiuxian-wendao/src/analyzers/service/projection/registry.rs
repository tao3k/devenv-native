use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;

use crate::analyzers::service::analyze_repository_from_config_with_registry;
use crate::analyzers::service::bootstrap_builtin_registry;

pub(super) fn with_repository_analysis<T, F>(
    repo_id: &str,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
    build: F,
) -> Result<T, RepoIntelligenceError>
where
    F: FnOnce(&RepositoryAnalysisOutput) -> Result<T, RepoIntelligenceError>,
{
    let analysis =
        analyze_repository_from_config_with_registry(repo_id, config_path, cwd, registry)?;
    build(&analysis)
}

pub(super) fn with_bootstrapped_repository_analysis<T, F>(
    repo_id: &str,
    config_path: Option<&Path>,
    cwd: &Path,
    build: F,
) -> Result<T, RepoIntelligenceError>
where
    F: FnOnce(&RepositoryAnalysisOutput) -> Result<T, RepoIntelligenceError>,
{
    let registry = bootstrap_builtin_registry()?;
    with_repository_analysis(repo_id, config_path, cwd, &registry, build)
}
