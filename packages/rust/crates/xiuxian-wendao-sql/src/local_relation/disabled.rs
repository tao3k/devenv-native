//! Feature-disabled `DuckDB` local relation engine placeholder.

use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;

use super::types::{LocalRelationEngine, LocalRelationEngineKind, LocalRelationRegistrationHint};

const DISABLED_MESSAGE: &str =
    "DuckDB local relation engine requires the `duckdb` feature on xiuxian-wendao-sql";

/// Feature-disabled `DuckDB` local relation engine placeholder.
#[derive(Debug, Clone, Copy, Default)]
pub struct FeatureDisabledDuckDbLocalRelationEngine;

impl FeatureDisabledDuckDbLocalRelationEngine {
    /// Create a fresh in-memory `DuckDB` local relation engine.
    ///
    /// # Errors
    ///
    /// Always returns an error when the `duckdb` feature is not enabled.
    pub fn new_in_memory() -> Result<Self, String> {
        Err(DISABLED_MESSAGE.to_string())
    }
}

#[async_trait]
impl LocalRelationEngine for FeatureDisabledDuckDbLocalRelationEngine {
    fn kind(&self) -> LocalRelationEngineKind {
        LocalRelationEngineKind::DuckDb
    }

    fn register_record_batches(
        &self,
        _table_name: &str,
        _schema: SchemaRef,
        _batches: Vec<RecordBatch>,
    ) -> Result<(), String> {
        Err(DISABLED_MESSAGE.to_string())
    }

    fn register_record_batches_with_hint(
        &self,
        _table_name: &str,
        _schema: SchemaRef,
        _batches: Vec<RecordBatch>,
        _hint: LocalRelationRegistrationHint,
    ) -> Result<(), String> {
        Err(DISABLED_MESSAGE.to_string())
    }

    async fn query_batches(&self, _sql: &str) -> Result<Vec<RecordBatch>, String> {
        Err(DISABLED_MESSAGE.to_string())
    }
}
