//! Agenda status state machine for task progression.

use serde::{Deserialize, Serialize};

/// Status of an agenda task.
/// Implements a strict state machine for task progression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Status {
    /// Task is planned but not started.
    #[default]
    Todo,
    /// Task is currently being worked on.
    Doing,
    /// Task is completed successfully.
    Done,
    /// Task is abandoned or postponed indefinitely.
    Canceled,
}

impl Status {
    /// Returns true if the status is a terminal state (Done or Canceled).
    #[must_use]
    pub fn is_terminal(self) -> bool {
        matches!(self, Status::Done | Status::Canceled)
    }
}
