//! Background repo indexing coordinator for Studio.

#[path = "repo_index/bootstrap.rs"]
mod bootstrap;
/// Benchmark helpers for repo-index performance probes.
#[cfg(feature = "performance")]
#[path = "repo_index/perf_support.rs"]
pub mod perf_support;
#[cfg(feature = "performance")]
#[path = "repo_index/policy.rs"]
mod policy;
#[path = "repo_index/state/mod.rs"]
mod state;
#[path = "repo_index/types.rs"]
mod types;

pub(crate) use bootstrap::start_repo_index_coordinator;
#[cfg(feature = "performance")]
pub(crate) use policy::repo_index_policy_debug_snapshot;
pub(crate) use state::RepoIndexCoordinator;
pub(crate) use types::RepoCodeDocument;
#[cfg(test)]
pub(crate) use types::RepoIndexSnapshot;
pub use types::{RepoIndexEntryStatus, RepoIndexPhase, RepoIndexRequest, RepoIndexStatusResponse};
