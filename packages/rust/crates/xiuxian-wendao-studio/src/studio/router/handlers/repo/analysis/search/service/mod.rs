//! Coordinates the Studio analysis search service branch and keeps its child modules behind one documented reasoning-tree boundary.

#[path = "execution.rs"]
mod execution;
#[path = "imports.rs"]
pub(crate) mod imports;
#[path = "typed.rs"]
pub(crate) mod typed;

#[cfg(test)]
#[path = "../../../../../../../../tests/unit/gateway/studio/router/handlers/repo/analysis/search/service.rs"]
mod tests;
