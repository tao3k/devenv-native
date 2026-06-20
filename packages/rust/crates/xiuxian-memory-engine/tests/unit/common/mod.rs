//! Shared test helpers for `xiuxian-memory-engine`.
//!
//! Store paths go under PRJ_CACHE_HOME/xiuxian-memory-engine per project conventions.

/// Path for test store under PRJ_CACHE/xiuxian-memory-engine.
///
/// Uses a unique suffix per call for parallel test isolation.
pub fn test_store_path(name: &str) -> String {
    let cache = xiuxian_config_core::ProjectDirs::cache_home();
    let base = cache.join("xiuxian-memory-engine").join(name);
    let unique = uuid::Uuid::new_v4();
    base.join(unique.to_string()).to_string_lossy().to_string()
}
