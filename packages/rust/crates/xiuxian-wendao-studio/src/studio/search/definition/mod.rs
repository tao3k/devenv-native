//! Logic for resolving the best semantic definition for a query.

mod filters;
mod resolve;
#[cfg(test)]
#[path = "../../../../tests/unit/gateway/studio/search/definition/mod.rs"]
mod tests;

pub use resolve::DefinitionResolveOptions;
pub(crate) use resolve::resolve_best_definition;
pub(crate) use resolve::resolve_definition_candidates;
