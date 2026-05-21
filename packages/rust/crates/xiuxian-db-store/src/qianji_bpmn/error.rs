//! Error model for Qianji BPMN workflow-state storage operations.

use super::{QianjiBpmnRecordKey, QianjiBpmnUpdatedAtMs};

/// Error returned by the BPMN `DuckDB` workflow data store.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum QianjiBpmnDataStoreError {
    /// Returned when a required record field is blank.
    #[error("BPMN DuckDB data-store field `{field}` cannot be blank")]
    BlankField {
        /// Stable field name.
        field: &'static str,
    },
    /// Returned when a timestamp cannot be represented by `DuckDB`'s integer lane.
    #[error(
        "BPMN DuckDB data-store record `{record_key}` timestamp {updated_at_ms} is out of range"
    )]
    TimestampOutOfRange {
        /// Record key whose timestamp failed conversion.
        record_key: QianjiBpmnRecordKey,
        /// Timestamp value in milliseconds.
        updated_at_ms: QianjiBpmnUpdatedAtMs,
    },
    /// Returned when JSON payload serialization or deserialization fails.
    #[error("BPMN DuckDB data-store codec operation `{operation}` failed: {message}")]
    Codec {
        /// Failing codec operation.
        operation: &'static str,
        /// Backend diagnostic message.
        message: String,
    },
    /// Returned when `DuckDB` storage cannot complete an operation.
    #[error("BPMN DuckDB data-store operation `{operation}` failed: {message}")]
    Storage {
        /// Failing storage operation.
        operation: &'static str,
        /// Backend diagnostic message.
        message: String,
    },
}
