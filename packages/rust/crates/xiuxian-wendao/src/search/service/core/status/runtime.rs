use super::helpers::{RepoCompactionRuntimeView, RepoPrewarmRuntimeView};
#[cfg(any(test, feature = "test-support"))]
#[cfg(test)]
use crate::repo_index::RepoIndexStatusResponse;
use crate::search::service::core::types::{
    RepoMaintenanceTask, RepoMaintenanceTaskKind, SearchPlaneService,
};
use crate::search::service::helpers::annotate_status_reason;
use crate::search::{SearchCorpusKind, SearchRepoReadPressure};

impl SearchPlaneService {
    /// Snapshot current multi-corpus status.
    #[must_use]
    pub fn status(&self) -> crate::search::SearchPlaneStatusSnapshot {
        let mut snapshot = self.coordinator.status();
        self.annotate_runtime_status_snapshot(&mut snapshot);
        snapshot
    }

    /// Snapshot current status after synchronizing repository runtime state.
    pub async fn status_with_repo_runtime(&self) -> crate::search::SearchPlaneStatusSnapshot {
        self.synchronize_repo_corpus_statuses_from_runtime().await;
        self.status()
    }

    #[cfg(test)]
    pub(crate) async fn status_with_repo_content(
        &self,
        repo_status: &RepoIndexStatusResponse,
    ) -> crate::search::SearchPlaneStatusSnapshot {
        self.synchronize_repo_runtime(repo_status);
        self.status_with_repo_runtime().await
    }

    fn annotate_runtime_status_snapshot(
        &self,
        snapshot: &mut crate::search::SearchPlaneStatusSnapshot,
    ) {
        snapshot.repo_read_pressure = Some(self.repo_read_pressure_snapshot());
        for status in &mut snapshot.corpora {
            self.annotate_runtime_status(status);
        }
    }

    fn repo_read_pressure_snapshot(&self) -> SearchRepoReadPressure {
        let dispatch = self
            .repo_search_dispatch
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let budget = u32::try_from(self.repo_search_read_concurrency_limit).unwrap_or(u32::MAX);
        let available =
            u32::try_from(self.repo_search_read_permits.available_permits()).unwrap_or(u32::MAX);
        let captured_at = dispatch.captured_at.clone();
        SearchRepoReadPressure {
            budget,
            in_flight: budget.saturating_sub(available),
            captured_at: captured_at.clone(),
            requested_repo_count: captured_at.as_ref().map(|_| dispatch.requested_repo_count),
            searchable_repo_count: captured_at.as_ref().map(|_| dispatch.searchable_repo_count),
            parallelism: captured_at.as_ref().map(|_| dispatch.parallelism),
            fanout_capped: dispatch.fanout_capped,
        }
    }

    pub(super) fn annotate_runtime_status(&self, status: &mut crate::search::SearchCorpusStatus) {
        if let Some(repo_prewarm) = self.repo_prewarm_runtime_view(status.corpus) {
            status.maintenance.prewarm_running |= repo_prewarm.is_running;
            status.maintenance.prewarm_queue_depth = status
                .maintenance
                .prewarm_queue_depth
                .max(repo_prewarm.queue_depth);
            match (
                status.maintenance.prewarm_queue_position,
                repo_prewarm.queue_position,
            ) {
                (None, Some(source_position)) => {
                    status.maintenance.prewarm_queue_position = Some(source_position);
                }
                (Some(target_position), Some(source_position))
                    if source_position < target_position =>
                {
                    status.maintenance.prewarm_queue_position = Some(source_position);
                }
                _ => {}
            }
        }
        if let Some(repo_compaction) = self.repo_compaction_runtime_view(status.corpus) {
            status.maintenance.compaction_running |= repo_compaction.is_running;
            status.maintenance.compaction_queue_depth = status
                .maintenance
                .compaction_queue_depth
                .max(repo_compaction.queue_depth);
            status.maintenance.compaction_queue_aged |= repo_compaction.queue_aged;
            match (
                status.maintenance.compaction_queue_position,
                repo_compaction.queue_position,
            ) {
                (None, Some(source_position)) => {
                    status.maintenance.compaction_queue_position = Some(source_position);
                }
                (Some(target_position), Some(source_position))
                    if source_position < target_position =>
                {
                    status.maintenance.compaction_queue_position = Some(source_position);
                }
                _ => {}
            }
        }
        status.last_query_telemetry = self.query_telemetry_for(status.corpus);
        annotate_status_reason(status);
    }

    fn repo_compaction_runtime_view(
        &self,
        corpus: SearchCorpusKind,
    ) -> Option<RepoCompactionRuntimeView> {
        if !matches!(
            corpus,
            SearchCorpusKind::RepoEntity | SearchCorpusKind::RepoContentChunk
        ) {
            return None;
        }
        let runtime = self
            .repo_maintenance
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let queue_depth = u32::try_from(
            runtime
                .queue
                .iter()
                .filter(|queued| {
                    matches!(
                        queued.task,
                        RepoMaintenanceTask::Compaction(ref task)
                            if task.corpus == corpus
                    )
                })
                .count(),
        )
        .unwrap_or(u32::MAX);
        let queue_position = runtime
            .queue
            .iter()
            .position(|queued| {
                matches!(
                    queued.task,
                    RepoMaintenanceTask::Compaction(ref task)
                        if task.corpus == corpus
                )
            })
            .and_then(|index| u32::try_from(index.saturating_add(1)).ok());
        let queue_aged = runtime
            .queue
            .iter()
            .find(|queued| {
                matches!(
                    queued.task,
                    RepoMaintenanceTask::Compaction(ref task)
                        if task.corpus == corpus
                )
            })
            .is_some_and(|queued| match &queued.task {
                RepoMaintenanceTask::Compaction(task) => Self::local_compaction_is_aged(
                    task.reason,
                    queued.enqueue_sequence,
                    runtime.next_enqueue_sequence,
                ),
                RepoMaintenanceTask::Prewarm(_) => false,
            });
        Some(RepoCompactionRuntimeView {
            is_running: runtime.active_task.as_ref().is_some_and(|task_key| {
                task_key.0 == corpus && matches!(task_key.3, RepoMaintenanceTaskKind::Compaction)
            }),
            queue_depth,
            queue_position,
            queue_aged,
        })
    }

    fn repo_prewarm_runtime_view(
        &self,
        corpus: SearchCorpusKind,
    ) -> Option<RepoPrewarmRuntimeView> {
        if !matches!(
            corpus,
            SearchCorpusKind::RepoEntity | SearchCorpusKind::RepoContentChunk
        ) {
            return None;
        }
        let runtime = self
            .repo_maintenance
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let queue_depth = u32::try_from(
            runtime
                .queue
                .iter()
                .filter(|queued| {
                    matches!(
                        queued.task,
                        RepoMaintenanceTask::Prewarm(ref task)
                            if task.corpus == corpus
                    )
                })
                .count(),
        )
        .unwrap_or(u32::MAX);
        let queue_position = runtime
            .queue
            .iter()
            .position(|queued| {
                matches!(
                    queued.task,
                    RepoMaintenanceTask::Prewarm(ref task)
                        if task.corpus == corpus
                )
            })
            .and_then(|index| u32::try_from(index.saturating_add(1)).ok());
        Some(RepoPrewarmRuntimeView {
            is_running: runtime.active_task.as_ref().is_some_and(|task_key| {
                task_key.0 == corpus && matches!(task_key.3, RepoMaintenanceTaskKind::Prewarm)
            }),
            queue_depth,
            queue_position,
        })
    }
}
