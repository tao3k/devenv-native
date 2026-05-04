//! Cargo entry point for `xiuxian-wendao-studio` performance tests.

#[path = "unit/lib_policy.rs"]
mod lib_policy;

rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = lib_policy::wendao_studio_harness_config()
);

#[cfg(feature = "performance")]
#[path = "performance/mod.rs"]
mod performance;
