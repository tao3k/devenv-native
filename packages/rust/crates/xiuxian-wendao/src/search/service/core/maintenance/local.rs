use xiuxian_db_store::VectorStoreError;

use super::helpers::{LOCAL_MAINTENANCE_SHUTDOWN_MESSAGE, PREWARM_ROW_LIMIT};
use crate::search::SearchCorpusKind;
use crate::search::service::core::types::SearchPlaneService;

impl SearchPlaneService {
    pub(crate) fn stop_background_maintenance(&self) {
        self.stop_local_maintenance();
        self.stop_repo_maintenance();
    }

    pub(crate) fn stop_local_maintenance(&self) {
        let mut runtime = self
            .local_maintenance
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        runtime.shutdown_requested = true;
    }

    pub(crate) async fn prewarm_epoch_table(
        &self,
        corpus: SearchCorpusKind,
        epoch: u64,
        projected_columns: &[&str],
    ) -> Result<(), VectorStoreError> {
        if self.local_maintenance_shutdown_requested() {
            return Err(VectorStoreError::General(
                LOCAL_MAINTENANCE_SHUTDOWN_MESSAGE.to_string(),
            ));
        }
        let table_names = self.local_epoch_table_names_for_reads(corpus, epoch);
        if table_names.is_empty() {
            return Err(VectorStoreError::TableNotFound(Self::table_name(
                corpus, epoch,
            )));
        }
        let _ = self.coordinator.mark_prewarm_running(corpus, epoch);
        let result = async {
            for table_name in table_names {
                self.prewarm_named_table(corpus, table_name.as_str(), projected_columns)
                    .await?;
            }
            Ok::<(), VectorStoreError>(())
        }
        .await;
        match result {
            Ok(()) => {
                let _ = self.coordinator.mark_prewarm_complete(corpus, epoch);
                Ok(())
            }
            Err(error) => {
                let _ = self.coordinator.clear_prewarm_running(corpus, epoch);
                Err(error)
            }
        }
    }

    pub(crate) async fn prewarm_named_table(
        &self,
        corpus: SearchCorpusKind,
        table_name: &str,
        projected_columns: &[&str],
    ) -> Result<(), VectorStoreError> {
        let parquet_path = self.named_table_parquet_path(corpus, table_name);
        if parquet_path.exists() {
            if (!corpus.is_repo_backed() && self.local_maintenance_shutdown_requested())
                || (corpus.is_repo_backed() && self.repo_maintenance_shutdown_requested())
            {
                return Err(VectorStoreError::General(if corpus.is_repo_backed() {
                    crate::search::service::core::maintenance::helpers::REPO_MAINTENANCE_SHUTDOWN_MESSAGE.to_string()
                } else {
                    LOCAL_MAINTENANCE_SHUTDOWN_MESSAGE.to_string()
                }));
            }
            let engine_table_name = Self::maintenance_engine_table_name(corpus, table_name);
            self.datafusion_query_engine
                .ensure_parquet_table_registered(
                    engine_table_name.as_str(),
                    parquet_path.as_path(),
                    &[],
                )
                .await?;
            let projection = if projected_columns.is_empty() {
                "*".to_string()
            } else {
                projected_columns.join(", ")
            };
            let query =
                format!("SELECT {projection} FROM {engine_table_name} LIMIT {PREWARM_ROW_LIMIT}");
            let _ = self
                .datafusion_query_engine
                .sql_batches(query.as_str())
                .await?;
            return Ok(());
        }

        Err(VectorStoreError::TableNotFound(table_name.to_string()))
    }

    pub(crate) fn local_maintenance_shutdown_requested(&self) -> bool {
        self.local_maintenance
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .shutdown_requested
    }
}
