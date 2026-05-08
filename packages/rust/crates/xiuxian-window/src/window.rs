//! Session window: bounded ring buffer of `TurnSlot`s.

use std::collections::VecDeque;

use crate::TurnSlot;

/// Owner-named session identifier for a bounded session window.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionWindowId(String);

impl SessionWindowId {
    /// Build a session-window identifier.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SessionWindowId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SessionWindowId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Owner-named checkpoint identifier for a turn slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionWindowCheckpointId(String);

impl SessionWindowCheckpointId {
    /// Build a checkpoint identifier.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Convert the identifier into its owned string representation.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<&str> for SessionWindowCheckpointId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SessionWindowCheckpointId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Named statistics for a bounded session window.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionWindowStats {
    /// Number of turns currently retained in the bounded window.
    pub total_turns: u64,
    /// Number of tool calls currently retained in the bounded window.
    pub total_tool_calls: u64,
    /// Number of slots currently used by the bounded ring.
    pub window_used: usize,
}

/// Bounded session window for recent turns. O(1) append, drop oldest when over capacity.
#[derive(Debug)]
pub struct SessionWindow {
    session_id: String,
    ring: VecDeque<TurnSlot>,
    max_turns: usize,
    total_tool_calls: u64,
}

impl SessionWindow {
    /// Create a session window with a fixed capacity.
    #[must_use]
    pub fn new(session_id: impl Into<SessionWindowId>, max_turns: usize) -> Self {
        let session_id = session_id.into();
        Self {
            session_id: session_id.as_str().to_string(),
            ring: VecDeque::with_capacity(max_turns.min(4096)),
            max_turns,
            total_tool_calls: 0,
        }
    }

    /// Append one turn. Drops oldest if over capacity.
    pub fn append_turn(
        &mut self,
        role: &str,
        content: &str,
        tool_count: u32,
        checkpoint_id: Option<SessionWindowCheckpointId>,
    ) {
        let mut slot = TurnSlot::new(role, content, tool_count);
        if let Some(id) = checkpoint_id {
            slot = slot.with_checkpoint(id);
        }
        self.total_tool_calls += u64::from(tool_count);
        self.ring.push_back(slot);
        while self.ring.len() > self.max_turns {
            if let Some(dropped) = self.ring.pop_front() {
                self.total_tool_calls = self
                    .total_tool_calls
                    .saturating_sub(u64::from(dropped.tool_count));
            }
        }
    }

    /// Last `max_turns` turns for context building (oldest to newest).
    #[must_use]
    pub fn get_recent_turns(&self, max_turns: usize) -> Vec<&TurnSlot> {
        let n = self.ring.len().min(max_turns);
        if n == 0 {
            return Vec::new();
        }
        let mut out: Vec<&TurnSlot> = self.ring.iter().rev().take(n).collect();
        out.reverse();
        out
    }

    /// Stats for consolidation trigger and UI.
    #[must_use]
    pub fn get_stats(&self) -> SessionWindowStats {
        SessionWindowStats {
            total_turns: self.ring.len() as u64,
            total_tool_calls: self.total_tool_calls,
            window_used: self.ring.len(),
        }
    }

    /// Drain the oldest `n` turns from the ring for consolidation. Returns drained slots and updates stats.
    /// Caller can summarise and store as episode then drop the returned slots.
    #[must_use]
    pub fn drain_oldest_turns(&mut self, n: usize) -> Vec<TurnSlot> {
        let take = n.min(self.ring.len());
        let mut out = Vec::with_capacity(take);
        for _ in 0..take {
            if let Some(slot) = self.ring.pop_front() {
                self.total_tool_calls = self
                    .total_tool_calls
                    .saturating_sub(u64::from(slot.tool_count));
                out.push(slot);
            }
        }
        out
    }

    /// Session identifier for this window.
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}
