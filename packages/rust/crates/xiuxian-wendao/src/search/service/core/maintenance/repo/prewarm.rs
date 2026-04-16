use chrono::Utc;
use xiuxian_vector_store::VectorStoreError;

use crate::search::service::core::maintenance::helpers::{
    repo_active_epoch, repo_runtime_status_for_record,
};
use crate::search::service::core::types::{
    RepoMaintenanceTask, RepoPrewarmTask, SearchPlaneService,
};
use crate::search::service::helpers::repo_corpus_staging_epoch;
use crate::search::{SearchCorpusKind, SearchMaintenanceStatus, SearchRepoCorpusRecord};

impl SearchPlaneService {
    pub(crate) async fn prewarm_repo_table(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
        table_name: &str,
        projected_columns: &[&str],
    ) -> Result<(), VectorStoreError> {
        if self.repo_maintenance_shutdown_requested() {
            return Err(VectorStoreError::General(
                crate::search::service::core::maintenance::helpers::REPO_MAINTENANCE_SHUTDOWN_MESSAGE.to_string(),
            ));
        }
        let task = RepoMaintenanceTask::Prewarm(RepoPrewarmTask {
            corpus,
            repo_id: repo_id.to_string(),
            table_name: table_name.to_string(),
            projected_columns: projected_columns
                .iter()
                .map(|column| (*column).to_string())
                .collect(),
        });
        let task_key = task.task_key();
        let (receiver, enqueued, start_worker) =
            self.register_repo_maintenance_task(task.clone(), true);
        if !enqueued
            && !self.repo_maintenance_shutdown_requested()
            && !self.repo_maintenance_task_is_live(&task_key)
        {
            self.complete_repo_maintenance_task(
                &task_key,
                &Err("stale repo maintenance claim without queued or active worker".to_string()),
            );
            return self.run_repo_maintenance_task(task).await;
        }
        self.ensure_repo_maintenance_worker(start_worker).await;
        self.await_repo_maintenance(receiver, &task_key).await
    }

    pub(crate) async fn mark_repo_prewarm_running(&self, corpus: SearchCorpusKind, repo_id: &str) {
        self.update_repo_prewarm_record(corpus, repo_id, |maintenance| {
            maintenance.prewarm_running = true;
        })
        .await;
    }

    pub(crate) async fn stop_repo_prewarm(&self, corpus: SearchCorpusKind, repo_id: &str) {
        self.update_repo_prewarm_record(corpus, repo_id, |maintenance| {
            maintenance.prewarm_running = false;
        })
        .await;
    }

    pub(super) async fn record_repo_corpus_prewarm(&self, corpus: SearchCorpusKind, repo_id: &str) {
        let mut record = self
            .repo_corpus_record_for_reads(corpus, repo_id)
            .await
            .unwrap_or_else(|| {
                SearchRepoCorpusRecord::new(
                    corpus,
                    repo_id.to_string(),
                    self.repo_runtime_state(repo_id)
                        .map(|state| Self::runtime_record_from_state(repo_id, &state)),
                    None,
                )
            });
        let mut repo_records = self.repo_corpus_snapshot_for_reads().await;
        repo_records.insert((corpus, repo_id.to_string()), record.clone());
        let relevant_records = repo_records
            .values()
            .filter(|candidate| candidate.corpus == corpus)
            .cloned()
            .collect::<Vec<_>>();
        let active_epoch = repo_active_epoch(corpus, relevant_records.as_slice());
        let runtime_statuses = relevant_records
            .iter()
            .filter_map(repo_runtime_status_for_record)
            .collect::<Vec<_>>();
        let prewarmed_epoch =
            repo_corpus_staging_epoch(corpus, &runtime_statuses, active_epoch).or(active_epoch);
        let mut maintenance = record.maintenance.unwrap_or_default();
        maintenance.prewarm_running = false;
        maintenance.last_prewarmed_at = Some(Utc::now().to_rfc3339());
        maintenance.last_prewarmed_epoch = prewarmed_epoch;
        record.maintenance = Some(maintenance);
        self.repo_corpus_records
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert((corpus, repo_id.to_string()), record.clone());
        self.cache.set_repo_corpus_record(&record).await;
        self.synchronize_repo_corpus_statuses_from_runtime().await;
    }

    async fn update_repo_prewarm_record<F>(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
        mutate: F,
    ) where
        F: FnOnce(&mut SearchMaintenanceStatus),
    {
        let mut record = self
            .repo_corpus_record_for_reads(corpus, repo_id)
            .await
            .unwrap_or_else(|| {
                SearchRepoCorpusRecord::new(
                    corpus,
                    repo_id.to_string(),
                    self.repo_runtime_state(repo_id)
                        .map(|state| Self::runtime_record_from_state(repo_id, &state)),
                    None,
                )
            });
        let mut maintenance = record.maintenance.unwrap_or_default();
        mutate(&mut maintenance);
        record.maintenance = Some(maintenance);
        self.repo_corpus_records
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert((corpus, repo_id.to_string()), record.clone());
        self.cache.set_repo_corpus_record(&record).await;
        self.synchronize_repo_corpus_statuses_from_runtime().await;
    }
}
