//! Logic for resolving the best semantic definition for a query.

mod filters;
mod resolve;
#[cfg(test)]
#[path = "../../../../tests/unit/gateway/studio/search/definition/mod.rs"]
mod tests;

pub(crate) use resolve::resolve_definition_candidates;
pub use resolve::{DefinitionResolveOptions, resolve_best_definition};
