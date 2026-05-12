//! `analyzers::cache::valkey` owns Wendao analyzers cache valkey behavior.

mod backend;
mod runtime;
mod scope;
mod storage;

#[cfg(all(test, feature = "zhenfa-router"))]
#[path = "../../../../tests/unit/analyzers/cache/valkey/mod.rs"]
mod tests;

pub use backend::ValkeyAnalysisCache;
use runtime::{ValkeyAnalysisCacheRuntime, resolve_valkey_analysis_cache_runtime};
pub(crate) use scope::{RepositoryAnalysisValkeyScope, RepositorySearchQueryValkeyScope};
use storage::{
    encode_analysis_payload, encode_search_query_payload, valkey_analysis_key,
    valkey_analysis_revision_key,
};
