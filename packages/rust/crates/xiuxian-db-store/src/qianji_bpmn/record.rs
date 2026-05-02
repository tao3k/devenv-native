//! Persisted Qianji BPMN workflow data record types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One bounded workflow-local data record stored by the BPMN `DuckDB` adapter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnDataRecord {
    /// Workflow instance identifier that owns the record.
    pub instance_id: String,
    /// Stable caller-owned key inside the workflow instance namespace.
    pub record_key: String,
    /// JSON-safe workflow payload.
    pub payload: Value,
    /// Caller-provided update timestamp in Unix milliseconds.
    pub updated_at_ms: u64,
}

impl QianjiBpmnDataRecord {
    /// Creates one workflow-local data record.
    #[must_use]
    pub fn new(
        instance_id: impl Into<String>,
        record_key: impl Into<String>,
        payload: Value,
        updated_at_ms: u64,
    ) -> Self {
        Self {
            instance_id: instance_id.into(),
            record_key: record_key.into(),
            payload,
            updated_at_ms,
        }
    }
}
