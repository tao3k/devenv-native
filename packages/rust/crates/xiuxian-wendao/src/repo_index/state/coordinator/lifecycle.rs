use std::sync::Arc;
use std::sync::atomic::Ordering;

use tokio::runtime::Handle;
use tokio::sync::OwnedSemaphorePermit;

use crate::analyzers::RepoIntelligenceError;

use crate::repo_index::state::coordinator::RepoIndexCoordinator;
use crate::repo_index::state::coordinator::handle::RepoIndexRuntimeHandle;

impl RepoIndexCoordinator {
    #[doc(hidden)]
    pub fn start(self: &Arc<Self>) {
        let Ok(handle) = Handle::try_current() else {
            return;
        };
        if self.started.swap(true, Ordering::SeqCst) {
            return;
        }
        *self
            .runtime_handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Some(RepoIndexRuntimeHandle::spawn(&handle, Arc::clone(self)));
    }

    /// Stop the background coordinator runtime task if it is active.
    pub fn stop(&self) {
        if let Some(runtime_handle) = self
            .runtime_handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        {
            runtime_handle.stop(self.notify.as_ref());
        }
    }

    /// Acquire one remote-sync concurrency permit for a repository task.
    ///
    /// # Errors
    ///
    /// Returns an analysis error when the semaphore has been closed.
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub async fn acquire_sync_permit(
        &self,
        repo_id: &str,
    ) -> Result<OwnedSemaphorePermit, RepoIntelligenceError> {
        Arc::clone(&self.sync_permits)
            .acquire_owned()
            .await
            .map_err(|_| RepoIntelligenceError::AnalysisFailed {
                message: format!(
                    "repo `{repo_id}` sync semaphore was closed while waiting to start remote sync"
                ),
            })
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/repo_index/state/coordinator/lifecycle.rs"]
mod tests;
