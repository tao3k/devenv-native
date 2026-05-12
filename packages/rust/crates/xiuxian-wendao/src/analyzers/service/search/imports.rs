//! `analyzers::service::search::imports` owns Wendao service search imports behavior.

use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
#[cfg(feature = "search-runtime")]
use crate::analyzers::cache::RepositorySearchArtifacts;
use crate::analyzers::service::{
    analyze_repository_from_config_with_registry, bootstrap_builtin_registry, import_match_score,
    normalized_rank_score,
};
use crate::analyzers::{ImportSearchHit, ImportSearchQuery, ImportSearchResult};

/// Build an import search result from normalized analysis records.
#[must_use]
pub fn build_import_search(
    query: &ImportSearchQuery,
    analysis: &RepositoryAnalysisOutput,
) -> ImportSearchResult {
    let limit = query.limit.max(1);
    let normalized_package = query.package.as_deref().map(str::to_ascii_lowercase);
    let normalized_module = query.module.as_deref().map(str::to_ascii_lowercase);

    let mut matches: Vec<(u8, _)> = analysis
        .imports
        .iter()
        .filter_map(|import| {
            let score = import_match_score(
                normalized_package.as_deref(),
                normalized_module.as_deref(),
                import,
            )?;
            Some((score, import))
        })
        .collect::<Vec<_>>();

    matches.sort_by(|(left_score, _left_import), (right_score, _right_import)| {
        left_score.cmp(right_score)
    });

    let selected = matches.into_iter().take(limit).collect::<Vec<_>>();
    let imports = selected
        .iter()
        .map(|(_score, import)| (*import).clone())
        .collect::<Vec<_>>();
    let import_hits = selected
        .into_iter()
        .enumerate()
        .map(|(index, (raw_score, import))| ImportSearchHit {
            import: import.clone(),
            score: Some(normalized_rank_score(raw_score, 3)),
            rank: Some(index + 1),
        })
        .collect::<Vec<_>>();

    ImportSearchResult {
        repo_id: query.repo_id.clone(),
        imports,
        import_hits,
    }
}
/// `build_import_search_with_artifacts` public function boundary for Wendao.

#[must_use]
#[cfg(feature = "search-runtime")]
pub fn build_import_search_with_artifacts(
    query: &ImportSearchQuery,
    analysis: &RepositoryAnalysisOutput,
    _artifacts: &RepositorySearchArtifacts,
) -> ImportSearchResult {
    build_import_search(query, analysis)
}

/// Load configuration, analyze one repository, and return matching imports.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn import_search_from_config_with_registry(
    query: &ImportSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<ImportSearchResult, RepoIntelligenceError> {
    let analysis =
        analyze_repository_from_config_with_registry(&query.repo_id, config_path, cwd, registry)?;
    Ok(build_import_search(query, &analysis))
}

/// Load configuration, analyze one repository, and return matching imports.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn import_search_from_config(
    query: &ImportSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<ImportSearchResult, RepoIntelligenceError> {
    let registry = bootstrap_builtin_registry()?;
    import_search_from_config_with_registry(query, config_path, cwd, &registry)
}
