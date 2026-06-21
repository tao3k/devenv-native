//! Unified local project-state path contracts.

pub mod git_utils;
mod paths;

pub use paths::{
    ProjectCacheRootConfig, STATE_STORE_DIR_NAME, STATE_STORE_DUCKDB_FILE_NAME, project_cache_root,
    project_cache_root_from_config, state_store_duckdb_path, state_store_root,
};
