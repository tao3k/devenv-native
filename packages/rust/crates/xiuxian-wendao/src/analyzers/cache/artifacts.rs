use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, OnceLock};

use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::cache::RepositoryAnalysisCacheKey;
use crate::analyzers::service::ExampleSearchMetadata;
use crate::analyzers::{ExampleRecord, ModuleRecord, ProjectedPageRecord, SymbolRecord};
use crate::search::SearchDocumentIndex;

/// Immutable search artifacts derived from one cached repository analysis snapshot.
#[derive(Clone)]
pub struct RepositorySearchArtifacts {
    /// Shared Tantivy index for module search.
    pub(crate) module_index: SearchDocumentIndex,
    /// Shared Tantivy index for symbol search.
    pub(crate) symbol_index: SearchDocumentIndex,
    /// Shared Tantivy index for example search.
    pub(crate) example_index: SearchDocumentIndex,
    /// Shared Tantivy index for projected-page search.
    pub(crate) projected_page_index: SearchDocumentIndex,
    /// Stable module lookup by identifier.
    pub(crate) modules_by_id: BTreeMap<String, ModuleRecord>,
    /// Stable symbol lookup by identifier.
    pub(crate) symbols_by_id: BTreeMap<String, SymbolRecord>,
    /// Stable example lookup by identifier.
    pub(crate) examples_by_id: BTreeMap<String, ExampleRecord>,
    /// Precomputed example metadata reused by search ranking.
    pub(crate) example_metadata: BTreeMap<String, ExampleSearchMetadata>,
    /// Stable projected-page lookup by page identifier.
    pub(crate) projected_pages_by_id: HashMap<String, ProjectedPageRecord>,
    /// Materialized projected pages reused by heuristic and lexical fallback.
    pub(crate) projected_pages: Vec<ProjectedPageRecord>,
}

type RepositorySearchArtifactsCache =
    BTreeMap<RepositoryAnalysisCacheKey, Arc<RepositorySearchArtifacts>>;

static REPOSITORY_SEARCH_ARTIFACTS_CACHE: OnceLock<Mutex<RepositorySearchArtifactsCache>> =
    OnceLock::new();

fn repository_search_artifacts_cache() -> &'static Mutex<RepositorySearchArtifactsCache> {
    REPOSITORY_SEARCH_ARTIFACTS_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Loads cached repository search artifacts if available.
///
/// # Errors
///
/// Returns an error when the in-memory cache lock is poisoned.
pub fn load_cached_repository_search_artifacts(
    key: &RepositoryAnalysisCacheKey,
) -> Result<Option<Arc<RepositorySearchArtifacts>>, RepoIntelligenceError> {
    repository_search_artifacts_cache()
        .lock()
        .map_err(|_| RepoIntelligenceError::AnalysisFailed {
            message: "repository search artifacts cache lock is poisoned".to_string(),
        })
        .map(|cache| cache.get(key).cloned())
}

/// Stores repository search artifacts in the cache and returns the shared handle.
///
/// # Errors
///
/// Returns an error when the in-memory cache lock is poisoned.
pub fn store_cached_repository_search_artifacts(
    key: RepositoryAnalysisCacheKey,
    artifacts: RepositorySearchArtifacts,
) -> Result<Arc<RepositorySearchArtifacts>, RepoIntelligenceError> {
    let artifacts = Arc::new(artifacts);
    repository_search_artifacts_cache()
        .lock()
        .map_err(|_| RepoIntelligenceError::AnalysisFailed {
            message: "repository search artifacts cache lock is poisoned".to_string(),
        })
        .map(|mut cache| {
            cache.insert(key, Arc::clone(&artifacts));
            artifacts
        })
}
