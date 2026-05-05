//! Wendao event-lake record types.

use chrono::{DateTime, Utc};
use serde_json::Value;

/// Single Wendao process or agent event prepared for event-lake ingestion.
#[derive(Debug, Clone, PartialEq)]
pub struct WendaoEventRecord {
    /// Tenant or workspace boundary for the event.
    pub tenant_id: String,
    /// Case, process, or workflow identifier.
    pub case_id: String,
    /// Stable event kind such as `bpmn.step`, `llm.call`, or `tool.call`.
    pub event_type: String,
    /// Compact JSON payload text owned by the event producer.
    pub payload_json: String,
    /// Event creation timestamp in UTC.
    pub created_at: DateTime<Utc>,
}

impl WendaoEventRecord {
    /// Build a Wendao event record from validated event fields.
    #[must_use]
    pub fn new(
        tenant_id: impl Into<String>,
        case_id: impl Into<String>,
        event_type: impl Into<String>,
        payload: &Value,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            case_id: case_id.into(),
            event_type: event_type.into(),
            payload_json: payload.to_string(),
            created_at,
        }
    }

    /// Build a Wendao event record from already serialized JSON payload text.
    ///
    /// # Errors
    ///
    /// Returns an error when `payload_json` is not valid JSON.
    pub fn from_payload_json(
        tenant_id: impl Into<String>,
        case_id: impl Into<String>,
        event_type: impl Into<String>,
        payload_json: impl Into<String>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, String> {
        let payload_json = payload_json.into();
        serde_json::from_str::<Value>(payload_json.as_str())
            .map_err(|error| format!("invalid Wendao event payload JSON: {error}"))?;
        Ok(Self::from_trusted_payload_json(
            tenant_id,
            case_id,
            event_type,
            payload_json,
            created_at,
        ))
    }

    pub(crate) fn from_trusted_payload_json(
        tenant_id: impl Into<String>,
        case_id: impl Into<String>,
        event_type: impl Into<String>,
        payload_json: impl Into<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            case_id: case_id.into(),
            event_type: event_type.into(),
            payload_json: payload_json.into(),
            created_at,
        }
    }

    /// Access the compact JSON payload text.
    #[must_use]
    pub fn payload_json(&self) -> &str {
        self.payload_json.as_str()
    }

    /// Parse the payload text into a JSON value.
    ///
    /// # Errors
    ///
    /// Returns an error when the stored payload text is not valid JSON.
    pub fn payload_value(&self) -> Result<Value, String> {
        serde_json::from_str(self.payload_json.as_str())
            .map_err(|error| format!("invalid Wendao event payload JSON: {error}"))
    }
}

/// Aggregate count for one Wendao event type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoEventTypeCount {
    /// Event kind.
    pub event_type: String,
    /// Number of rows observed for this event kind.
    pub count: i64,
}
