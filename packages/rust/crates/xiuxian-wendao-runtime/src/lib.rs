//! Generic host behavior for the Wendao package split.
//!
//! Ownership rule:
//! - put runtime config resolution, settings merge, transport negotiation,
//!   client/server construction, and runtime artifact helpers here
//! - do not put stable contract ownership or Wendao business-domain logic here
//!
//! This crate sits between `xiuxian-wendao-core` and `xiuxian-wendao`: it owns
//! deployment-dependent host behavior, while the main Wendao crate owns graph,
//! retrieval, storage, and other product semantics.

/// Runtime-owned artifact render helpers.
pub mod artifacts;
/// Runtime-owned live link-graph config records and resolvers.
pub mod config;
/// Runtime-owned config settings merge, override, and parsing helpers.
pub mod settings;
/// Transport negotiation and client-construction helpers.
pub mod transport;

#[cfg(test)]
#[path = "../tests/unit/lib/mod.rs"]
mod tests;
