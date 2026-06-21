//! Shared test helpers for `xiuxian-memory-engine`.
//!
//! Store paths go under the project-local state cache per project conventions.

/// Path for test store under the project-local xiuxian-memory-engine state root.
///
/// Uses a unique suffix per call for parallel test isolation.
pub fn test_store_path(name: &str) -> String {
    let base = xiuxian_db_store::state::state_store_root()
        .join("xiuxian-memory-engine")
        .join(name);
    let unique = uuid::Uuid::new_v4();
    base.join(unique.to_string()).to_string_lossy().to_string()
}
