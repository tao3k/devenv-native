use std::path::Path;
use std::sync::Arc;

use arrow::datatypes::DataType;
use arrow::record_batch::RecordBatch;
use datafusion::dataframe::DataFrame;
use datafusion::execution::context::SessionContext;
use datafusion::prelude::{ParquetReadOptions, SessionConfig};

use crate::VectorStoreError;

/// Partition column metadata used when registering partitioned Parquet search-plane tables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchEnginePartitionColumn {
    /// Partition column name.
    pub name: String,
    /// Partition column type.
    pub data_type: DataType,
}

impl SearchEnginePartitionColumn {
    /// Create a new partition-column declaration.
    #[must_use]
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
        }
    }
}

/// Project-scoped `DataFusion` execution context for Wendao search-plane reads.
#[derive(Clone)]
pub struct SearchEngineContext {
    session: Arc<SessionContext>,
}

impl Default for SearchEngineContext {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchEngineContext {
    /// Create a new search-engine context with statistics collection enabled.
    #[must_use]
    pub fn new() -> Self {
        let mut config = SessionConfig::new();
        config.options_mut().execution.collect_statistics = true;
        Self::new_with_config(config)
    }

    /// Create a new search-engine context from an existing `DataFusion` session config.
    #[must_use]
    pub fn new_with_config(config: SessionConfig) -> Self {
        let mut config = config;
        // DataFusion 53 can produce invalid repartitioned window/sort plans for
        // repo-content ranking queries; keep the safer non-repartitioned
        // strategy for search reads.
        config.options_mut().optimizer.repartition_windows = false;
        config.options_mut().optimizer.repartition_sorts = false;
        Self {
            session: Arc::new(SessionContext::new_with_config(config)),
        }
    }

    /// Access the underlying `DataFusion` session.
    #[must_use]
    pub fn session(&self) -> &SessionContext {
        self.session.as_ref()
    }

    /// Register a Parquet table or partitioned directory for query execution.
    ///
    /// # Errors
    ///
    /// Returns an error when the table path is unreadable or `DataFusion` rejects the registration.
    pub async fn register_parquet_table(
        &self,
        table_name: &str,
        table_path: &Path,
        partition_columns: &[SearchEnginePartitionColumn],
    ) -> Result<(), VectorStoreError> {
        let options = ParquetReadOptions::default().table_partition_cols(
            partition_columns
                .iter()
                .map(|column| (column.name.clone(), column.data_type.clone()))
                .collect(),
        );
        self.session
            .register_parquet(table_name, &table_path.to_string_lossy(), options)
            .await?;
        Ok(())
    }

    /// Register a Parquet table only when the table name is not already present.
    ///
    /// # Errors
    ///
    /// Returns an error when the table path is unreadable or `DataFusion` rejects the registration.
    pub async fn ensure_parquet_table_registered(
        &self,
        table_name: &str,
        table_path: &Path,
        partition_columns: &[SearchEnginePartitionColumn],
    ) -> Result<(), VectorStoreError> {
        if self.table(table_name).await.is_ok() {
            return Ok(());
        }
        self.register_parquet_table(table_name, table_path, partition_columns)
            .await
    }

    /// Resolve a registered table as a `DataFusion` dataframe.
    ///
    /// # Errors
    ///
    /// Returns an error when the named table has not been registered.
    pub async fn table(&self, table_name: &str) -> Result<DataFrame, VectorStoreError> {
        self.session.table(table_name).await.map_err(Into::into)
    }

    /// Execute a SQL query and collect Arrow batches.
    ///
    /// # Errors
    ///
    /// Returns an error when SQL planning or execution fails.
    pub async fn sql_batches(&self, sql: &str) -> Result<Vec<RecordBatch>, VectorStoreError> {
        let dataframe = self.session.sql(sql).await?;
        self.collect_dataframe(dataframe).await
    }

    /// Collect a `DataFusion` dataframe into Arrow batches.
    ///
    /// # Errors
    ///
    /// Returns an error when the dataframe execution fails.
    pub async fn collect_dataframe(
        &self,
        dataframe: DataFrame,
    ) -> Result<Vec<RecordBatch>, VectorStoreError> {
        dataframe.collect().await.map_err(Into::into)
    }
}
