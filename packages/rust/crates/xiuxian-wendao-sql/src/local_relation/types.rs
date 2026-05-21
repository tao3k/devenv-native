//! Engine-neutral local relation traits and registration evidence.

use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;

/// Stable internal engine kinds for bounded local relation execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalRelationEngineKind {
    /// DuckDB-backed execution.
    DuckDb,
}

impl LocalRelationEngineKind {
    /// Stable explain or telemetry label for the active local relation engine.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DuckDb => "duckdb",
        }
    }
}

/// Stable bounded local relation materialization states for explain-facing metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalRelationMaterializationState {
    /// The engine materialized the relation into engine-owned table storage.
    Materialized,
    /// The engine kept the relation virtual over caller-owned Arrow batches.
    Virtual,
}

impl LocalRelationMaterializationState {
    /// Stable explain or telemetry label for the relation materialization state.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Materialized => "materialized",
            Self::Virtual => "virtual",
        }
    }
}

/// Narrow caller hint for one request-scoped local relation registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalRelationRegistrationHint {
    /// Use the engine default registration policy.
    Default,
    /// The caller expects repeated relation queries inside the current request scope.
    RepeatedUse,
}

/// Narrow local relation-engine seam for bounded in-process analytics.
#[async_trait]
pub trait LocalRelationEngine: Send + Sync {
    /// Report the active bounded local relation-engine kind.
    fn kind(&self) -> LocalRelationEngineKind;

    /// Register one set of in-memory record batches as a queryable table.
    ///
    /// # Errors
    ///
    /// Returns an error when the batches cannot be normalized into a queryable
    /// in-memory table or when registration fails.
    fn register_record_batches(
        &self,
        table_name: &str,
        schema: SchemaRef,
        batches: Vec<RecordBatch>,
    ) -> Result<(), String>;

    /// Register one set of in-memory record batches as a queryable table with a usage hint.
    ///
    /// # Errors
    ///
    /// Returns an error when the batches cannot be normalized into a queryable
    /// in-memory table or when registration fails.
    fn register_record_batches_with_hint(
        &self,
        table_name: &str,
        schema: SchemaRef,
        batches: Vec<RecordBatch>,
        hint: LocalRelationRegistrationHint,
    ) -> Result<(), String> {
        let _ = hint;
        self.register_record_batches(table_name, schema, batches)
    }

    /// Report the registration strategy used for one registered relation when exposed.
    #[must_use]
    fn relation_registration_strategy(&self, _table_name: &str) -> Option<&'static str> {
        None
    }

    /// Report the materialization state used for one registered relation when exposed.
    #[must_use]
    fn relation_materialization_state(
        &self,
        _table_name: &str,
    ) -> Option<LocalRelationMaterializationState> {
        None
    }

    /// Report the peak temp-storage bytes observed for the last bounded local query.
    #[must_use]
    fn last_query_temp_storage_peak_bytes(&self) -> Option<u64> {
        None
    }

    /// Execute one SQL query and collect Arrow batches.
    ///
    /// # Errors
    ///
    /// Returns an error when planning or execution fails.
    async fn query_batches(&self, sql: &str) -> Result<Vec<RecordBatch>, String>;
}
