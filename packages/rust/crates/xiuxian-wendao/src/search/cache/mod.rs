mod config;
mod construction;
mod fingerprints;
mod keys;
mod reads;
mod reads_blocking;
mod runtime;
#[cfg(test)]
#[path = "../../../tests/unit/search/cache/mod.rs"]
mod tests;
mod types;
mod writes;

#[cfg(any(test, feature = "test-support"))]
pub(crate) use config::SearchPlaneCacheConfig;
pub use config::SearchPlaneCacheTtl;
pub(crate) use fingerprints::SearchPlaneFileFingerprintScope;
pub use runtime::resolve_search_plane_cache_connection_target;
pub(crate) use types::SearchPlaneCache;
