//! Agenda entry record for actionable cultivation tasks.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Priority;

/// Represents an actionable item in the cultivation agenda (a 'Vow').
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgendaEntry {
    /// Unique identifier for the task.
    pub id: Uuid,
    /// Human-readable title of the task.
    pub title: String,
    /// Task importance level.
    pub priority: Priority,
    /// Current "heat" or urgency (0.0 to 1.0).
    pub heat: f32,
    /// Number of times this task has been carried over to the next day.
    pub carryover_count: u32,
    /// Optional scheduled time for the task (UTC).
    pub scheduled_at: Option<DateTime<Utc>>,
    /// Optional expected duration in minutes.
    pub duration_minutes: Option<u32>,
    /// Whether the user has been reminded about this task.
    pub reminded: bool,
}

impl AgendaEntry {
    /// Creates a new agenda entry with default priority and heat.
    #[must_use]
    pub fn new(title: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            priority: Priority::Medium,
            heat: 0.5,
            carryover_count: 0,
            scheduled_at: None,
            duration_minutes: None,
            reminded: false,
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/agenda/entry.rs"]
mod tests;
