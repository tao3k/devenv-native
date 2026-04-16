use chrono::Utc;
use xiuxian_vector_store::TableInfo;

use crate::search::service::core::types::{RepoCompactionTask, SearchPlaneService};
use crate::search::{SearchMaintenanceStatus, SearchRepoCorpusRecord, SearchRepoPublicationRecord};

impl SearchPlaneService {
    pub(crate) fn next_repo_publication_maintenance(
        &self,
        previous_record: Option<&SearchRepoCorpusRecord>,
        next_row_count: u64,
    ) -> SearchMaintenanceStatus {
        let mut maintenance = previous_record
            .and_then(|record| record.maintenance.clone())
            .unwrap_or_default();
        let publish_count = maintenance.publish_count_since_compaction.saturating_add(1);
        maintenance.publish_count_since_compaction = publish_count;
        maintenance.compaction_running = false;
        maintenance.compaction_pending = self.coordinator.maintenance_policy().should_compact(
            publish_count,
            maintenance.last_compacted_row_count,
            next_row_count,
        );
        maintenance
    }

    pub(crate) async fn schedule_repo_compaction_if_needed(&self, record: &SearchRepoCorpusRecord) {
        let Some(compaction_task) = self.repo_compaction_task(record) else {
            return;
        };
        let task =
            crate::search::service::core::types::RepoMaintenanceTask::Compaction(compaction_task);
        let (_receiver, enqueued, start_worker) = self.register_repo_maintenance_task(task, false);
        if !enqueued {
            return;
        }
        self.ensure_repo_maintenance_worker(start_worker).await;
    }

    fn repo_compaction_task(&self, record: &SearchRepoCorpusRecord) -> Option<RepoCompactionTask> {
        let publication = record.publication.as_ref()?;
        if publication.is_parquet_query_readable() {
            return None;
        }
        let maintenance = record.maintenance.as_ref()?;
        if !maintenance.compaction_pending {
            return None;
        }
        let reason = self.coordinator.maintenance_policy().compaction_reason(
            maintenance.publish_count_since_compaction,
            maintenance.last_compacted_row_count,
            publication.row_count,
        )?;
        Some(RepoCompactionTask {
            corpus: record.corpus,
            repo_id: record.repo_id.clone(),
            publication_id: publication.publication_id.clone(),
            table_name: publication.table_name.clone(),
            row_count: publication.row_count,
            reason,
        })
    }

    pub(crate) async fn mark_repo_compaction_running(&self, task: &RepoCompactionTask) -> bool {
        self.update_repo_compaction_record(task, |_publication, maintenance| {
            maintenance.compaction_running = true;
        })
        .await
    }

    pub(crate) async fn stop_repo_compaction(
        &self,
        task: &RepoCompactionTask,
        keep_pending: bool,
    ) -> bool {
        self.update_repo_compaction_record(task, |_publication, maintenance| {
            maintenance.compaction_running = false;
            maintenance.compaction_pending = keep_pending;
        })
        .await
    }

    pub(crate) async fn run_repo_compaction_task(&self, task: RepoCompactionTask) {
        let store = match self.open_store(task.corpus).await {
            Ok(store) => store,
            Err(error) => {
                log::warn!(
                    "search-plane repo compaction failed to open store for {} repo {} table {}: {}",
                    task.corpus,
                    task.repo_id,
                    task.table_name,
                    error
                );
                let _ = self.stop_repo_compaction(&task, true).await;
                return;
            }
        };
        match store.compact(task.table_name.as_str()).await {
            Ok(_) => match store.get_table_info(task.table_name.as_str()).await {
                Ok(table_info) => {
                    let _ = self.complete_repo_compaction(&task, &table_info).await;
                }
                Err(error) => {
                    log::warn!(
                        "search-plane repo compaction failed to inspect {} repo {} table {} after compact: {}",
                        task.corpus,
                        task.repo_id,
                        task.table_name,
                        error
                    );
                    let _ = self.stop_repo_compaction(&task, true).await;
                }
            },
            Err(error) => {
                log::warn!(
                    "search-plane repo compaction failed for {} repo {} table {}: {}",
                    task.corpus,
                    task.repo_id,
                    task.table_name,
                    error
                );
                let _ = self.stop_repo_compaction(&task, true).await;
            }
        }
    }

    async fn complete_repo_compaction(
        &self,
        task: &RepoCompactionTask,
        table_info: &TableInfo,
    ) -> bool {
        self.update_repo_compaction_record(task, |publication, maintenance| {
            publication.fragment_count =
                u64::try_from(table_info.fragment_count).unwrap_or(u64::MAX);
            maintenance.compaction_running = false;
            maintenance.compaction_pending = false;
            maintenance.publish_count_since_compaction = 0;
            maintenance.last_compacted_at = Some(Utc::now().to_rfc3339());
            maintenance.last_compaction_reason = Some(task.reason.as_str().to_string());
            maintenance.last_compacted_row_count = Some(table_info.num_rows);
        })
        .await
    }

    async fn update_repo_compaction_record<F>(&self, task: &RepoCompactionTask, mutate: F) -> bool
    where
        F: FnOnce(&mut SearchRepoPublicationRecord, &mut SearchMaintenanceStatus),
    {
        let key = (task.corpus, task.repo_id.clone());
        let Some(mut record) = self
            .repo_corpus_record_for_reads(task.corpus, task.repo_id.as_str())
            .await
        else {
            return false;
        };
        let Some(publication) = record.publication.as_mut() else {
            return false;
        };
        if publication.publication_id != task.publication_id {
            return false;
        }
        let mut maintenance = record.maintenance.clone().unwrap_or_default();
        mutate(publication, &mut maintenance);
        record.maintenance = Some(maintenance);
        self.repo_corpus_records
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(key, record.clone());
        self.cache.set_repo_corpus_record(&record).await;
        self.synchronize_repo_corpus_statuses_from_runtime().await;
        true
    }
}
