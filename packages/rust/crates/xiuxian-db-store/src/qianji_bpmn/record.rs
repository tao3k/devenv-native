//! Persisted Qianji BPMN workflow data record types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

macro_rules! qianji_bpmn_string_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Borrows the serialized value.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::ops::Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                self.as_str()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }
    };
}

qianji_bpmn_string_type!(
    /// Workflow instance identifier in the Qianji BPMN `DuckDB` data store.
    QianjiBpmnInstanceId
);
qianji_bpmn_string_type!(
    /// Workflow-local data record key in the Qianji BPMN `DuckDB` data store.
    QianjiBpmnRecordKey
);

/// Caller-provided update timestamp in Unix milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QianjiBpmnUpdatedAtMs(u64);

impl QianjiBpmnUpdatedAtMs {
    /// Returns the raw millisecond timestamp.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for QianjiBpmnUpdatedAtMs {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for QianjiBpmnUpdatedAtMs {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.get())
    }
}

/// One bounded workflow-local data record stored by the BPMN `DuckDB` adapter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnDataRecord {
    /// Workflow instance identifier that owns the record.
    pub instance_id: QianjiBpmnInstanceId,
    /// Stable caller-owned key inside the workflow instance namespace.
    pub record_key: QianjiBpmnRecordKey,
    /// JSON-safe workflow payload.
    pub payload: Value,
    /// Caller-provided update timestamp in Unix milliseconds.
    pub updated_at_ms: QianjiBpmnUpdatedAtMs,
}

impl QianjiBpmnDataRecord {
    /// Creates one workflow-local data record.
    #[must_use]
    pub fn new(
        instance_id: impl Into<QianjiBpmnInstanceId>,
        record_key: impl Into<QianjiBpmnRecordKey>,
        payload: Value,
        updated_at_ms: impl Into<QianjiBpmnUpdatedAtMs>,
    ) -> Self {
        Self {
            instance_id: instance_id.into(),
            record_key: record_key.into(),
            payload,
            updated_at_ms: updated_at_ms.into(),
        }
    }
}
