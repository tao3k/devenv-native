#[cfg(feature = "performance")]
mod diagnostics;
mod handle;
mod hydration;
mod lifecycle;
mod queue;
#[path = "runtime/mod.rs"]
mod runtime;
mod status;
mod types;

#[cfg(test)]
pub(crate) use runtime::PreparedIncrementalAnalysis;
pub use types::RepoIndexCoordinator;
