//! In-memory and Valkey-backed analysis cache for repository intelligence.

mod analysis;
#[cfg(feature = "search-runtime")]
mod artifacts;
#[path = "identity/mod.rs"]
mod identity;
mod keys;
mod query;
#[path = "valkey/mod.rs"]
mod valkey;

#[cfg(all(
    test,
    feature = "julia",
    feature = "repo-lexical-index",
    feature = "search-runtime"
))]
#[path = "../../../tests/unit/analyzers/cache/mod.rs"]
mod tests;

pub use analysis::load_cached_repository_analysis;
#[cfg(feature = "search-runtime")]
pub use analysis::load_cached_repository_analysis_for_revision;
pub use analysis::store_cached_repository_analysis;
#[cfg(feature = "search-runtime")]
pub use artifacts::{
    RepositorySearchArtifacts, load_cached_repository_search_artifacts,
    store_cached_repository_search_artifacts,
};
#[cfg(feature = "search-runtime")]
pub(crate) use identity::{
    FingerprintMode, analysis_fingerprint_mode, change_affects_analysis_identity,
    plugin_ids_support_semantic_owner_reuse, semantic_fingerprint_for_file,
};
pub use keys::build_repository_analysis_cache_key;
pub use keys::{RepositoryAnalysisCacheKey, RepositorySearchQueryCacheKey};
pub use query::{load_cached_repository_search_result, store_cached_repository_search_result};
pub(crate) use valkey::{
    RepositoryAnalysisValkeyScope, RepositorySearchQueryValkeyScope, ValkeyAnalysisCache,
};
