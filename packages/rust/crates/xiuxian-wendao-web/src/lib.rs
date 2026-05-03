//! Wendao Flight and gRPC transport boundary.
//!
//! This crate intentionally stays small: it exposes only transport contracts
//! and service wiring for high-throughput Flight/gRPC callers. Studio, HTTP,
//! `OpenAPI`, parser, analyzer, and repository-domain behavior live outside this
//! package boundary.

#[cfg(test)]
#[path = "../tests/unit/lib_policy.rs"]
mod rust_project_harness_gate;

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = rust_project_harness_gate::wendao_web_harness_config()
);

/// Flight and gRPC transport contracts for Wendao.
#[cfg(feature = "transport")]
pub mod transport;
