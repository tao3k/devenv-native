//! Julia runtime contracts and feature-scoped adapters.
//!
//! Wendao integration lives behind the `wendao` feature and consumes inert
//! Julia fact catalogs from `xiuxian-polyglot-orchestrator`.

#[cfg(feature = "wendao")]
/// Wendao-facing Julia runtime facts and contract identities.
pub mod wendao;

#[cfg(test)]
#[path = "../tests/unit/lib/mod.rs"]
mod tests;
