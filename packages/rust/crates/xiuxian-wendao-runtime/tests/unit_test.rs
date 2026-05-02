//! Root unit-test harness for `xiuxian-wendao-runtime`.

rust_lang_project_harness::rust_project_harness_gate!();

#[path = "unit/artifacts_openapi.rs"]
mod artifacts_openapi;

#[path = "unit/artifacts_zhixing.rs"]
mod artifacts_zhixing;
