use std::sync::Arc;

use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use datafusion::datasource::MemTable;
use datafusion::execution::context::SessionContext;
use datafusion::prelude::SessionConfig;

use super::types::{
    LocalRelationEngine, LocalRelationEngineKind, LocalRelationMaterializationState,
};

/// Current active bounded local relation engine backed by request-scoped `DataFusion`.
#[derive(Clone)]
pub struct DataFusionLocalRelationEngine {
    session: SessionContext,
}

impl DataFusionLocalRelationEngine {
    /// Create a new request-scoped `DataFusion` engine with `information_schema` enabled.
    #[must_use]
    pub fn new_with_information_schema() -> Self {
        let mut config = SessionConfig::new().with_information_schema(true);
        config.options_mut().execution.collect_statistics = true;
        Self {
            session: SessionContext::new_with_config(config),
        }
    }

    /// Access the underlying `DataFusion` session.
    #[must_use]
    pub fn session(&self) -> &SessionContext {
        &self.session
    }
}

#[async_trait]
impl LocalRelationEngine for DataFusionLocalRelationEngine {
    fn kind(&self) -> LocalRelationEngineKind {
        LocalRelationEngineKind::DataFusion
    }

    fn register_record_batches(
        &self,
        table_name: &str,
        schema: SchemaRef,
        batches: Vec<RecordBatch>,
    ) -> Result<(), String> {
        let batches = if batches.is_empty() {
            vec![RecordBatch::new_empty(Arc::clone(&schema))]
        } else {
            batches
        };
        let table = MemTable::try_new(schema, vec![batches]).map_err(|error| {
            format!("failed to build DataFusion memtable `{table_name}`: {error}")
        })?;
        self.session
            .register_table(table_name, Arc::new(table))
            .map_err(|error| {
                format!("failed to register DataFusion table `{table_name}`: {error}")
            })?;
        Ok(())
    }

    fn relation_materialization_state(
        &self,
        table_name: &str,
    ) -> Option<LocalRelationMaterializationState> {
        let _ = table_name;
        Some(LocalRelationMaterializationState::Materialized)
    }

    async fn query_batches(&self, sql: &str) -> Result<Vec<RecordBatch>, String> {
        self.session
            .sql(sql)
            .await
            .map_err(|error| format!("failed to plan bounded SQL query: {error}"))?
            .collect()
            .await
            .map_err(|error| format!("failed to execute bounded SQL query: {error}"))
    }
}
