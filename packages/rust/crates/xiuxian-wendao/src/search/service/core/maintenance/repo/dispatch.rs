use xiuxian_db_store::VectorStoreError;

use crate::search::service::core::types::{RepoMaintenanceTask, SearchPlaneService};

impl SearchPlaneService {
    pub(crate) async fn run_repo_maintenance_task(
        &self,
        task: RepoMaintenanceTask,
    ) -> Result<(), VectorStoreError> {
        match task {
            RepoMaintenanceTask::Prewarm(task) => {
                let projected_columns = task
                    .projected_columns
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                self.prewarm_named_table(task.corpus, task.table_name.as_str(), &projected_columns)
                    .await?;
                self.record_repo_corpus_prewarm(task.corpus, task.repo_id.as_str())
                    .await;
                Ok(())
            }
            RepoMaintenanceTask::Compaction(task) => {
                self.run_repo_compaction_task(task).await;
                Ok(())
            }
        }
    }
}
