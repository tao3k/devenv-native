//! Cargo entry point for `xiuxian-wendao-studio` unit tests.

#[path = "unit/lib_policy.rs"]
mod lib_policy;
#[path = "unit/namespace.rs"]
mod namespace;
#[cfg(feature = "studio")]
#[path = "unit/studio_search_index_api.rs"]
mod studio_search_index_api;
