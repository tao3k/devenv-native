use tokio::sync::oneshot;
use xiuxian_db_store::VectorStoreError;

use crate::search::service::core::types::{
    RepoMaintenanceTask, RepoMaintenanceTaskKey, RepoMaintenanceTaskResult, SearchPlaneService,
};

impl SearchPlaneService {
    pub(crate) async fn ensure_repo_maintenance_worker(&self, start_worker: bool) {
        if !start_worker {
            return;
        }
        if self.repo_maintenance_shutdown_requested() {
            let mut runtime = self
                .repo_maintenance
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            runtime.worker_running = false;
            return;
        }
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let service = self.clone();
            let worker_handle = handle.spawn(async move {
                service.run_repo_maintenance_worker().await;
            });
            self.repo_maintenance
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .worker_handle = Some(worker_handle);
        } else {
            self.run_repo_maintenance_worker().await;
        }
    }

    async fn run_repo_maintenance_worker(&self) {
        loop {
            let queued = {
                let mut runtime = self
                    .repo_maintenance
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if runtime.shutdown_requested {
                    runtime.active_task = None;
                    runtime.worker_running = false;
                    runtime.worker_handle = None;
                    break;
                }
                if let Some(queued) = runtime.queue.pop_front() {
                    runtime.active_task = Some(queued.task.task_key());
                    queued
                } else {
                    runtime.active_task = None;
                    runtime.worker_running = false;
                    runtime.worker_handle = None;
                    break;
                }
            };
            let task_key = queued.task.task_key();
            if let RepoMaintenanceTask::Prewarm(task) = &queued.task {
                self.mark_repo_prewarm_running(task.corpus, task.repo_id.as_str())
                    .await;
            }
            if let RepoMaintenanceTask::Compaction(task) = &queued.task
                && !self.mark_repo_compaction_running(task).await
            {
                self.complete_repo_maintenance_task(
                    &task_key,
                    &Err("failed to mark repo compaction as running".to_string()),
                );
                continue;
            }
            let result = self
                .run_repo_maintenance_task(queued.task.clone())
                .await
                .map_err(|error| error.to_string());
            if let (RepoMaintenanceTask::Prewarm(task), Err(_)) = (&queued.task, &result) {
                self.stop_repo_prewarm(task.corpus, task.repo_id.as_str())
                    .await;
            }
            self.complete_repo_maintenance_task(&task_key, &result);
        }
    }

    pub(crate) async fn await_repo_maintenance(
        &self,
        receiver: Option<oneshot::Receiver<RepoMaintenanceTaskResult>>,
        task_key: &RepoMaintenanceTaskKey,
    ) -> Result<(), VectorStoreError> {
        let Some(receiver) = receiver else {
            return Ok(());
        };
        match receiver.await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(message)) => Err(VectorStoreError::General(message)),
            Err(_) => {
                self.complete_repo_maintenance_task(
                    task_key,
                    &Err("repo maintenance worker dropped before completing task".to_string()),
                );
                Err(VectorStoreError::General(
                    "repo maintenance worker dropped before completing task".to_string(),
                ))
            }
        }
    }

    pub(crate) fn complete_repo_maintenance_task(
        &self,
        task_key: &RepoMaintenanceTaskKey,
        result: &RepoMaintenanceTaskResult,
    ) {
        let waiters = {
            let mut runtime = self
                .repo_maintenance
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            runtime.in_flight.remove(task_key);
            if runtime.active_task.as_ref() == Some(task_key) {
                runtime.active_task = None;
            }
            runtime.waiters.remove(task_key).unwrap_or_default()
        };
        for waiter in waiters {
            let _ = waiter.send(result.clone());
        }
    }

    pub(crate) fn repo_maintenance_task_is_live(&self, task_key: &RepoMaintenanceTaskKey) -> bool {
        let runtime = self
            .repo_maintenance
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        runtime.active_task.as_ref() == Some(task_key)
            || runtime
                .queue
                .iter()
                .any(|queued| &queued.task.task_key() == task_key)
    }

    pub(crate) fn repo_maintenance_shutdown_requested(&self) -> bool {
        self.repo_maintenance
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .shutdown_requested
    }
}
