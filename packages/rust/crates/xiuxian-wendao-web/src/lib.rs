//! Web-facing Wendao gateway namespace.
//!
//! This crate is the migration boundary for HTTP, `OpenAPI`, Studio router, and
//! web DTO surfaces. The first slice re-exports the existing
//! `xiuxian_wendao::gateway` implementation so callers can adopt the clearer
//! package name before implementation modules move.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!();

/// Stable OpenAPI contract exports for the Wendao gateway.
pub mod openapi;

#[cfg(feature = "studio")]
pub use xiuxian_wendao::gateway;
#[cfg(feature = "studio")]
pub use xiuxian_wendao::gateway::studio;
