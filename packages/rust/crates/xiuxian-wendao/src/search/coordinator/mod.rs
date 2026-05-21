//! `search::coordinator` owns Wendao search coordinator behavior.

mod build;
mod maintenance;
mod state;
mod types;

pub use state::SearchPlaneCoordinator;
pub(crate) use types::SearchCompactionReason;
pub use types::{BeginBuildDecision, SearchBuildLease};

#[cfg(test)]
#[path = "../../../tests/unit/search/coordinator/mod.rs"]
mod tests;
