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
    /// JSON payload owned by the event producer.
    pub payload: Value,
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
        payload: Value,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            case_id: case_id.into(),
            event_type: event_type.into(),
            payload,
            created_at,
        }
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
