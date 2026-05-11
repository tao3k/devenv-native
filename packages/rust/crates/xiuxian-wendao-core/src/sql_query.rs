//! JSON payload contracts for request-scoped SQL query responses.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

macro_rules! sql_query_value_type {
    ($name:ident, $inner:ty, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name($inner);

        impl $name {
            /// Returns the wrapped value.
            #[must_use]
            pub fn get(&self) -> &$inner {
                &self.0
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        impl std::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                self.get()
            }
        }

        impl PartialEq<$inner> for $name {
            fn eq(&self, other: &$inner) -> bool {
                self.get() == other
            }
        }

        impl PartialOrd<$inner> for $name {
            fn partial_cmp(&self, other: &$inner) -> Option<std::cmp::Ordering> {
                self.get().partial_cmp(other)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.get().fmt(formatter)
            }
        }
    };
}

sql_query_value_type!(
    SqlByteCount,
    u64,
    "Byte count reported by SQL query execution."
);
sql_query_value_type!(
    SqlDurationMs,
    u64,
    "Duration in milliseconds reported by SQL query execution."
);
sql_query_value_type!(
    SqlMaterializationState,
    String,
    "Stable bounded local relation materialization-state label."
);
sql_query_value_type!(
    SqlDataTypeLabel,
    String,
    "Stable SQL result data-type label."
);

/// Stable metadata returned for one request-scoped SQL query.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SqlQueryMetadata {
    /// Stable table name for the request-scoped table inventory catalog.
    pub catalog_table_name: String,
    /// Stable table name for the request-scoped column inventory catalog.
    pub column_catalog_table_name: String,
    /// Stable table name for the request-scoped logical-view source catalog.
    pub view_source_catalog_table_name: String,
    /// Whether the SQL engine exposes `information_schema`.
    pub supports_information_schema: bool,
    /// Stable SQL-visible object names registered for the current request.
    pub registered_tables: Vec<String>,
    /// Count of registered SQL-visible tables for the current request.
    pub registered_table_count: usize,
    /// Count of registered logical views for the current request.
    pub registered_view_count: usize,
    /// Count of registered SQL-visible columns for the current request.
    pub registered_column_count: usize,
    /// Count of logical-view source rows for the current request.
    pub registered_view_source_count: usize,
    /// Count of result batches returned by the query.
    pub result_batch_count: usize,
    /// Count of rows returned across all result batches.
    pub result_row_count: usize,
    /// Count of array-backed bytes registered into the bounded local relation
    /// before query execution when the caller exposes that detail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registered_input_bytes: Option<SqlByteCount>,
    /// Count of array-backed bytes returned across all result batches when the
    /// caller exposes that detail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_bytes: Option<SqlByteCount>,
    /// Stable bounded local relation materialization-state label when the
    /// caller exposes that detail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_relation_materialization_state: Option<SqlMaterializationState>,
    /// Peak temp-storage bytes observed for the last bounded local query when
    /// the caller exposes that detail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_temp_storage_peak_bytes: Option<SqlByteCount>,
    /// Stable local relation-engine label for bounded local analytics when the
    /// caller exposes that detail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_relation_engine: Option<String>,
    /// Stable `DuckDB` registration-strategy label when the bounded local engine
    /// exposes that detail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duckdb_registration_strategy: Option<String>,
    /// Count of input batches registered into the bounded local relation before
    /// query execution when the caller exposes that detail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registered_input_batch_count: Option<usize>,
    /// Count of rows registered into the bounded local relation before query
    /// execution when the caller exposes that detail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registered_input_row_count: Option<usize>,
    /// Milliseconds spent registering the bounded local relation before query
    /// execution when the caller exposes that detail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registration_time_ms: Option<SqlDurationMs>,
    /// Milliseconds spent executing the bounded local SQL statement when the
    /// caller exposes that detail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_query_execution_time_ms: Option<SqlDurationMs>,
}

/// Stable description of one SQL result column.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SqlColumnPayload {
    /// Column name as exposed to the caller.
    pub name: String,
    /// Stable `Arrow` or `DataFusion` data-type label.
    pub data_type: SqlDataTypeLabel,
    /// Whether the column accepts null values.
    pub nullable: bool,
}

/// Stable JSON-friendly representation of one SQL result batch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SqlBatchPayload {
    /// Row count for this batch.
    pub row_count: usize,
    /// Ordered column descriptors for this batch schema.
    pub columns: Vec<SqlColumnPayload>,
    /// Ordered row payloads for this batch.
    pub rows: Vec<Map<String, Value>>,
}

/// Stable JSON-friendly representation of one SQL query result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SqlQueryPayload {
    /// Request-scoped discovery and result metadata.
    pub metadata: SqlQueryMetadata,
    /// Materialized result batches.
    pub batches: Vec<SqlBatchPayload>,
}
