use anyhow::Result;

use crate::agent::{Agent, SessionContextMode, SessionContextWindowInfo};
use crate::observability::SessionEvent;

impl Agent {
    /// Inspect active context window counters for this session.
    ///
    /// # Errors
    /// Returns an error when reading session counters from bounded/unbounded backends fails.
    pub async fn inspect_context_window(
        &self,
        session_id: impl AsRef<str>,
    ) -> Result<SessionContextWindowInfo> {
        let session_id = session_id.as_ref();
        if let Some(ref w) = self.bounded_session {
            let stats = w.get_stats(session_id).await?;
            let turn_count = stats.map_or(0, |stats| stats.turn_count);
            let total_tool_calls = stats.map_or(0, |stats| stats.total_tool_calls);
            let window_slots = stats.map_or(0, |stats| stats.ring_len);
            let summary_segments = w.get_summary_segment_count(session_id).await?;
            let info = SessionContextWindowInfo {
                mode: SessionContextMode::Bounded,
                messages: window_slots,
                summary_segments,
                window_turns: Some(usize::try_from(turn_count).unwrap_or(usize::MAX)),
                window_slots: Some(window_slots),
                total_tool_calls: Some(total_tool_calls),
            };
            tracing::debug!(
                event = SessionEvent::BoundedStatsLoaded.as_str(),
                session_id,
                mode = "bounded",
                messages = info.messages,
                summary_segments = info.summary_segments,
                window_turns = ?info.window_turns,
                window_slots = ?info.window_slots,
                total_tool_calls = ?info.total_tool_calls,
                "session context window inspected"
            );
            return Ok(info);
        }

        let message_count = self.session.len(session_id).await?;
        let info = SessionContextWindowInfo {
            mode: SessionContextMode::Unbounded,
            messages: message_count,
            summary_segments: 0,
            window_turns: None,
            window_slots: None,
            total_tool_calls: None,
        };
        tracing::debug!(
            event = SessionEvent::SessionMessagesLoaded.as_str(),
            session_id,
            mode = "unbounded",
            messages = info.messages,
            summary_segments = info.summary_segments,
            "session context window inspected"
        );
        Ok(info)
    }
}
