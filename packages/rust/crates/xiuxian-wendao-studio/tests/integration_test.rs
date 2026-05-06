//! Cargo entry point for `xiuxian-wendao-studio` integration tests.

#[path = "unit/lib_policy.rs"]
mod lib_policy;

rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = lib_policy::wendao_studio_harness_config()
);

#[cfg(feature = "zhenfa-router")]
#[path = "integration/semantic_scope_provider.rs"]
mod semantic_scope_provider;
