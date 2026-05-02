//! Session context snapshot and memory window types.

use serde::{Deserialize, Serialize};
use xiuxian_window::TurnSlot;

use crate::session::{ChatMessage, SessionSummarySegment};

/// Lightweight stats for a session context buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionContextStats {
    /// Message count represented by this context.
    pub messages: usize,
    /// Summary segment count represented by this context.
    pub summary_segments: usize,
}

/// Persisted snapshot metadata for session context backups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionContextSnapshotInfo {
    /// Backed-up message count.
    pub messages: usize,
    /// Backed-up summary segment count.
    pub summary_segments: usize,
    /// Backup timestamp in Unix milliseconds.
    pub saved_at_unix_ms: Option<u64>,
    /// Age of the backup in seconds.
    pub saved_age_secs: Option<u64>,
}

/// Session context operation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionContextMode {
    /// Bounded turn window mode.
    Bounded,
    /// Unbounded append-only mode.
    Unbounded,
}

/// Runtime info for current session context window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionContextWindowInfo {
    /// Active context mode.
    pub mode: SessionContextMode,
    /// Message count currently visible in window.
    pub messages: usize,
    /// Summary segment count currently visible in window.
    pub summary_segments: usize,
    /// Configured max turns for bounded mode.
    pub window_turns: Option<usize>,
    /// Actual turn-slot count in bounded mode.
    pub window_slots: Option<usize>,
    /// Total tool calls observed in window, when tracked.
    pub total_tool_calls: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct SessionContextBackupMetadata {
    pub(super) messages: usize,
    pub(super) summary_segments: usize,
    pub(super) saved_at_unix_ms: u64,
}

#[derive(Clone, Default)]
pub(super) struct SessionContextBackup {
    pub(super) messages: Vec<ChatMessage>,
    pub(super) summary_segments: Vec<SessionSummarySegment>,
    pub(super) window_slots: Vec<TurnSlot>,
}

impl SessionContextBackup {
    pub(super) fn stats(&self) -> SessionContextStats {
        let messages = if self.window_slots.is_empty() {
            self.messages.len()
        } else {
            self.window_slots.len()
        };
        SessionContextStats {
            messages,
            summary_segments: self.summary_segments.len(),
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.messages.is_empty() && self.summary_segments.is_empty() && self.window_slots.is_empty()
    }
}
