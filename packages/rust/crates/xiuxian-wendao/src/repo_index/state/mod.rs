mod collect;
mod coordinator;
mod filters;
mod fingerprint;
mod language;
mod task;

#[cfg(feature = "performance")]
pub(crate) use collect::{collect_code_documents, collect_incremental_code_documents};
pub(crate) use coordinator::RepoIndexCoordinator;
#[cfg(feature = "performance")]
pub(crate) use task::{
    repo_index_analysis_timeout, repo_index_sync_requeue_attempt_limit, repo_index_sync_timeout,
};

#[cfg(test)]
#[path = "../../../tests/unit/repo_index/state/mod.rs"]
mod tests;
