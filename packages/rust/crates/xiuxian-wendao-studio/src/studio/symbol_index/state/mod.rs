//! Coordinates the Studio studio symbol index state branch and keeps its child modules behind one documented reasoning-tree boundary.

mod build;
mod coordinator;
mod metadata;

#[cfg(test)]
#[path = "../../../../tests/unit/gateway/studio/symbol_index/state/mod.rs"]
mod tests;

pub(crate) use build::maybe_spawn_build;
pub(crate) use coordinator::SymbolIndexCoordinator;
pub(crate) use metadata::{fingerprint_projects, timestamp_now};
