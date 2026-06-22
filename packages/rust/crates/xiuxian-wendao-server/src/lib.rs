//! Wendao Flight and gRPC transport boundary.
//!
//! This crate intentionally stays small: it exposes only transport contracts
//! and service wiring for high-throughput Flight/gRPC callers. Studio, HTTP,
//! `OpenAPI`, parser, analyzer, and repository-domain behavior live outside this
//! package boundary.

/// Flight and gRPC transport contracts for Wendao.
#[cfg(feature = "transport")]
pub mod transport;
