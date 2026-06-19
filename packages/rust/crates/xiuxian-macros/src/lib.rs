//! # xiuxian-macros
//!
//! Common procedural macros for omni Rust crates.
//!
//! ## Macros
//!
//! ### Code Generation
//! - [`patterns!`] - Generate pattern constants for symbol extraction
//! - [`topics!`] - Generate topic/event constants
//! - [`py_from!`] - Generate `PyO3` From implementations
//! - [`env_non_empty!`] - Read a trimmed non-empty environment variable as `Option<String>`
//! - [`string_first_non_empty!`] - Resolve the first non-empty string candidate
//! - [`project_config_paths!`] - Build system/user/env layered config candidate paths
//! - [`crate_resources_dir!`] - Embed the calling crate's local `resources/` tree
//! - [`embed_utf8_dir!`] - Embed a UTF-8 resources directory as sorted file pairs
//!
//! ### Testing Utilities
//! - [`temp_dir!`] - Create a temporary directory for tests
//! - [`assert_timing!`] - Assert timing constraint for benchmarks
//! - [`bench_case!`] - Create a benchmark test case

mod constants;
mod embed_utf8_dir;
mod env;
#[macro_use]
mod facade;
mod py;
mod resources;
mod testing;
mod xiuxian_config;

#[cfg(test)]
crate_testing_source_gate!();

/// Attribute macro for loading cascading config into a struct.
#[proc_macro_attribute]
pub fn xiuxian_config(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    xiuxian_config::expand(attr, item)
}

/// Embed a directory of UTF-8 files as sorted `(path, content)` pairs.
#[proc_macro]
pub fn embed_utf8_dir(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    embed_utf8_dir::expand(input)
}

/// Generate pattern constants for symbol extraction.
#[proc_macro]
pub fn patterns(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    constants::expand_patterns(input)
}

/// Generate topic/event constants.
#[proc_macro]
pub fn topics(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    constants::expand_topics(input)
}

/// Generate `PyO3` From implementations for wrapper types.
#[proc_macro]
pub fn py_from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    py::expand_from(input)
}

/// Read an environment variable and return `Option<String>` when non-empty after trim.
#[proc_macro]
pub fn env_non_empty(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    env::expand_non_empty(input)
}

/// Resolve the first non-empty string from ordered `Option<&str>`-like candidates.
#[proc_macro]
pub fn string_first_non_empty(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    env::expand_first_non_empty(input)
}

/// Embed the calling crate's local `resources/` directory.
#[proc_macro]
pub fn crate_resources_dir(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    resources::expand_crate_dir(input)
}

/// Build layered config candidate paths for a config filename.
#[proc_macro]
pub fn project_config_paths(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    resources::expand_project_config_paths(input)
}

/// Create a temporary directory for tests.
#[proc_macro]
pub fn temp_dir(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    testing::expand_temp_dir(input)
}

/// Assert timing constraint for benchmarks.
#[proc_macro]
pub fn assert_timing(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    testing::expand_assert_timing(input)
}

/// Create a benchmark test case with timing.
#[proc_macro]
pub fn bench_case(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    testing::expand_bench_case(input)
}
