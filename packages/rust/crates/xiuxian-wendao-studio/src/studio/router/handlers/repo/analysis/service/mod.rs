//! Coordinates the Studio repo analysis service branch and keeps its child modules behind one documented reasoning-tree boundary.

#[path = "coverage.rs"]
pub(crate) mod coverage;
#[path = "overview.rs"]
pub(crate) mod overview;

#[cfg(test)]
#[path = "../../../../../../../tests/unit/gateway/studio/router/handlers/repo/analysis/service.rs"]
mod tests;
