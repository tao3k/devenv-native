//! `analyzers::cache::analysis` owns Wendao analyzers cache analysis behavior.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::cache::RepositoryAnalysisCacheKey;

type RepositoryAnalysisCache = BTreeMap<RepositoryAnalysisCacheKey, RepositoryAnalysisOutput>;

static REPOSITORY_ANALYSIS_CACHE: OnceLock<Mutex<RepositoryAnalysisCache>> = OnceLock::new();

fn repository_analysis_cache() -> &'static Mutex<RepositoryAnalysisCache> {
    REPOSITORY_ANALYSIS_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Loads a cached analysis result if available.
///
/// # Errors
///
/// Returns an error when the in-memory cache lock is poisoned.
pub fn load_cached_repository_analysis(
    key: &RepositoryAnalysisCacheKey,
) -> Result<Option<RepositoryAnalysisOutput>, RepoIntelligenceError> {
    repository_analysis_cache()
        .lock()
        .map_err(|_| RepoIntelligenceError::AnalysisFailed {
            message: "repository analysis cache lock is poisoned".to_string(),
        })
        .map(|cache| cache.get(key).cloned())
}

/// Loads a cached analysis result by revision lookup when the analysis
/// identity has already changed.
///
/// # Errors
///
/// Returns an error when the in-memory cache lock is poisoned.
#[cfg(feature = "search-runtime")]
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn load_cached_repository_analysis_for_revision(
    repo_id: &str,
    checkout_root: &str,
    plugin_ids: &[String],
    revision: &str,
) -> Result<Option<RepositoryAnalysisOutput>, RepoIntelligenceError> {
    repository_analysis_cache()
        .lock()
        .map_err(|_| RepoIntelligenceError::AnalysisFailed {
            message: "repository analysis cache lock is poisoned".to_string(),
        })
        .map(|cache| {
            cache.iter().find_map(|(key, output)| {
                key.matches_revision_lookup(repo_id, checkout_root, plugin_ids, revision)
                    .then_some(output.clone())
            })
        })
}

/// Stores an analysis result in the cache.
///
/// # Errors
///
/// Returns an error when the in-memory cache lock is poisoned.
pub fn store_cached_repository_analysis(
    key: RepositoryAnalysisCacheKey,
    output: &RepositoryAnalysisOutput,
) -> Result<(), RepoIntelligenceError> {
    repository_analysis_cache()
        .lock()
        .map_err(|_| RepoIntelligenceError::AnalysisFailed {
            message: "repository analysis cache lock is poisoned".to_string(),
        })
        .map(|mut cache| {
            cache.insert(key, output.clone());
        })
}
