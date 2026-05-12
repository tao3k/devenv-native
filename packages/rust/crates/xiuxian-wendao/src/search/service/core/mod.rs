//! `search::service::core` owns Wendao search service core behavior.

mod cache_keys;
#[path = "construction/mod.rs"]
mod construction;
mod file_fingerprints;
mod ingest;
#[path = "local_runtime/mod.rs"]
mod local_runtime;
#[path = "maintenance/mod.rs"]
mod maintenance;
mod markdown_snapshot;
mod publication;
mod queries;
mod repeat_work;
#[path = "repo_runtime/mod.rs"]
mod repo_runtime;
mod source_snapshot;
#[path = "status/mod.rs"]
mod status;
mod telemetry;
mod types;

pub use repeat_work::SearchBuildRepeatWorkTelemetry;
#[cfg(test)]
pub(crate) use types::RepoMaintenanceTaskKind;
#[cfg(test)]
pub(crate) use types::RepoPrewarmTask;
pub(crate) use types::RepoRuntimeState;
pub use types::SearchPlaneService;
#[cfg(test)]
pub(crate) use types::{QueuedRepoMaintenanceTask, RepoCompactionTask, RepoMaintenanceTask};
pub use types::{RepoSearchAvailability, RepoSearchPublicationState, RepoSearchQueryCacheKeyInput};
