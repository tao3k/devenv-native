//! Cargo entry point for `xiuxian-wendao-studio` unit tests.

#[path = "unit/lib_policy.rs"]
mod lib_policy;

rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = lib_policy::wendao_studio_harness_config()
);

#[cfg(feature = "contracts")]
#[path = "unit/contracts_dependency_boundary/mod.rs"]
mod contracts_dependency_boundary;
#[cfg(feature = "contracts")]
#[path = "unit/contracts_routes.rs"]
mod contracts_routes;
#[cfg(feature = "contracts")]
#[path = "unit/contracts_types.rs"]
mod contracts_types;
#[path = "unit/namespace.rs"]
mod namespace;
#[cfg(feature = "studio")]
#[path = "unit/studio_search_index_api.rs"]
mod studio_search_index_api;
