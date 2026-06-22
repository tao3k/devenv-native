//! Unified cascading configuration kernel and local path helpers.
//!
//! ## Macros
//!
//! - `crate_resources_dir!` - Expands to an embedded `include_dir::Dir`
//!   rooted at the absolute `resources/` directory for the crate that invokes
//!   the macro.
//! - `toml_first_env!` - Resolves a TOML-owned value first, then falls back to
//!   a precedence-ordered env lookup chain.
//! - `first_some!` - Resolves the first `Some(...)` candidate from an ordered
//!   precedence chain.

pub use xiuxian_macros::crate_resources_dir;

mod cache;
mod error;
mod macros;
mod paths;
mod resolve;
mod spec;
mod test_support;

pub(crate) use cache::{build_file_stamps, cache_key, store_cached_merged, try_get_cached_merged};
pub use error::ConfigCoreError;
pub use paths::{
    ProjectDirs, ProjectDirsConfig, absolutize_path, normalize_config_home, resolve_cache_home,
    resolve_cache_home_from_value, resolve_config_home, resolve_data_home, resolve_path_from_value,
    resolve_project_root, resolve_project_root_or_cwd, resolve_project_root_or_cwd_from_value,
    resolve_runtime_dir, resolve_runtime_dir_from_value,
};
pub use resolve::{
    NamedScalarValue, first_non_empty_lookup, first_non_empty_named_lookup,
    load_toml_value_with_imports, load_toml_value_with_imports_and_paths, lookup_bool_flag,
    lookup_parsed, lookup_positive_parsed, merge_toml_values, parse_bool_flag, parse_positive,
    parse_trimmed, resolve_and_load, resolve_and_load_with_paths, resolve_and_merge_toml,
    resolve_and_merge_toml_with_paths, toml_first_env_parsed, toml_first_env_string,
    toml_first_named_string, trimmed_non_empty,
};
pub use spec::{ArrayMergeStrategy, ConfigCascadeSpec};
pub use test_support::resolve_home_from_value;
