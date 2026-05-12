//! `repo_index::state::task` owns Wendao repo index state task behavior.

mod adaptive;
mod machine;
mod policy;
mod types;

#[cfg(feature = "performance")]
pub(crate) use adaptive::AdaptiveConcurrencyDebugSnapshot;
pub(crate) use adaptive::{AdaptiveConcurrencyController, AdaptiveConcurrencySnapshot};
#[cfg(any(test, feature = "performance"))]
pub(crate) use policy::repo_index_sync_requeue_attempt_limit;
pub(crate) use policy::{
    repo_index_analysis_timeout, repo_index_sync_concurrency_limit, repo_index_sync_timeout,
    should_penalize_adaptive_concurrency, should_retry_sync_failure,
};
pub(crate) use types::{RepoIndexTask, RepoIndexTaskPriority, RepoTaskFeedback, RepoTaskOutcome};
