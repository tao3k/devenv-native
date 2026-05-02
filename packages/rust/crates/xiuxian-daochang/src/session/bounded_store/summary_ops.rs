use std::collections::VecDeque;

use anyhow::{Context, Result};

use crate::observability::SessionEvent;

use super::BoundedSessionStore;
use crate::session::SessionSummarySegment;

impl BoundedSessionStore {
    /// Append a compact summary segment produced during consolidation.
    ///
    /// # Errors
    /// Returns an error when appending summary segment to Valkey fails.
    pub async fn append_summary_segment(
        &self,
        session_id: &str,
        segment: SessionSummarySegment,
    ) -> Result<()> {
        let mut segment = segment;
        segment.summary = truncate_to_chars(&segment.summary, self.summary_max_chars);
        if segment.summary.is_empty() {
            return Ok(());
        }

        if let Some(ref redis) = self.redis {
            redis
                .append_summary_segment(session_id, self.summary_max_segments, &segment)
                .await
                .with_context(|| {
                    format!("valkey bounded summary append failed for session_id={session_id}")
                })?;
            tracing::debug!(
                event = SessionEvent::BoundedSummarySegmentAppended.as_str(),
                session_id,
                chars = segment.summary.chars().count(),
                max_segments = self.summary_max_segments,
                backend = "valkey",
                "bounded session summary segment appended"
            );
            return Ok(());
        }

        let mut g = self.summaries.write().await;
        let queue = g
            .entry(session_id.to_string())
            .or_insert_with(|| VecDeque::with_capacity(self.summary_max_segments));
        queue.push_back(segment);
        while queue.len() > self.summary_max_segments {
            let _ = queue.pop_front();
        }
        tracing::debug!(
            event = SessionEvent::BoundedSummarySegmentAppended.as_str(),
            session_id,
            max_segments = self.summary_max_segments,
            current_segments = queue.len(),
            backend = "memory",
            "bounded session summary segment appended"
        );
        Ok(())
    }

    /// Get the most recent compact summary segments for prompt context injection.
    ///
    /// # Errors
    /// Returns an error when loading summary segments from Valkey fails.
    pub async fn get_recent_summary_segments(
        &self,
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<SessionSummarySegment>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        if let Some(ref redis) = self.redis {
            let segments = redis
                .get_recent_summary_segments(session_id, limit)
                .await
                .with_context(|| {
                    format!("valkey bounded summary read failed for session_id={session_id}")
                })?;
            tracing::debug!(
                event = SessionEvent::BoundedSummarySegmentsLoaded.as_str(),
                session_id,
                requested_limit = limit,
                loaded_segments = segments.len(),
                backend = "valkey",
                "bounded session summary segments loaded"
            );
            return Ok(segments);
        }
        let g = self.summaries.read().await;
        let Some(queue) = g.get(session_id) else {
            return Ok(Vec::new());
        };
        let take = queue.len().min(limit);
        let mut out = queue.iter().rev().take(take).cloned().collect::<Vec<_>>();
        out.reverse();
        tracing::debug!(
            event = SessionEvent::BoundedSummarySegmentsLoaded.as_str(),
            session_id,
            requested_limit = limit,
            loaded_segments = out.len(),
            backend = "memory",
            "bounded session summary segments loaded"
        );
        Ok(out)
    }

    /// Count compact summary segments for the session without loading full contents.
    ///
    /// # Errors
    /// Returns an error when reading summary segment count from Valkey fails.
    pub async fn get_summary_segment_count(&self, session_id: &str) -> Result<usize> {
        if let Some(ref redis) = self.redis {
            let segment_count = redis.get_summary_len(session_id).await.with_context(|| {
                format!("valkey bounded summary count failed for session_id={session_id}")
            })?;
            tracing::debug!(
                event = SessionEvent::BoundedSummarySegmentsLoaded.as_str(),
                session_id,
                loaded_segments = segment_count,
                backend = "valkey",
                count_only = true,
                "bounded session summary segment count loaded"
            );
            return Ok(segment_count);
        }

        let g = self.summaries.read().await;
        let segment_count = g.get(session_id).map_or(0, VecDeque::len);
        tracing::debug!(
            event = SessionEvent::BoundedSummarySegmentsLoaded.as_str(),
            session_id,
            loaded_segments = segment_count,
            backend = "memory",
            count_only = true,
            "bounded session summary segment count loaded"
        );
        Ok(segment_count)
    }
}

fn truncate_to_chars(input: &str, max_chars: usize) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() || max_chars == 0 {
        return String::new();
    }
    let char_count = trimmed.chars().count();
    if char_count <= max_chars {
        return trimmed.to_string();
    }
    let keep = max_chars.saturating_sub(3);
    let mut out = trimmed.chars().take(keep).collect::<String>();
    out.push_str("...");
    out
}
