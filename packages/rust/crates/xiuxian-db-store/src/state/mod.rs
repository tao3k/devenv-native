//! Unified user-local Artisan state path contracts.

pub mod git_utils;
mod paths;

pub use paths::{
    ARTISAN_STATE_ROOT_DIR_NAME, ArtisanStateRootConfig, STATE_STORE_DIR_NAME,
    STATE_STORE_DUCKDB_FILE_NAME, artisan_state_root, artisan_state_root_from_config,
    state_store_duckdb_path, state_store_root,
};
