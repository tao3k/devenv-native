use anyhow::{Context, Result};
use xiuxian_window::TurnSlot;

use crate::observability::SessionEvent;
use crate::session::{BoundedSessionStore, ChatMessage};

use super::turn_slots_to_messages;

impl BoundedSessionStore {
    /// Returns recent turns as `ChatMessage` rows (`role` + `content` only) for LLM context.
    /// Oldest first.
    ///
    /// # Errors
    /// Returns an error when loading bounded window slots from Valkey fails.
    pub async fn get_recent_messages(
        &self,
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<ChatMessage>> {
        let limit_slots = limit.saturating_mul(2);
        if let Some(ref redis) = self.redis {
            let slots = redis
                .get_recent_window_slots(session_id, limit_slots)
                .await
                .with_context(|| {
                    format!("valkey bounded session read failed for session_id={session_id}")
                })?;
            let messages = turn_slots_to_messages(&slots);
            tracing::debug!(
                event = SessionEvent::BoundedRecentMessagesLoaded.as_str(),
                session_id,
                requested_turns = limit,
                loaded_messages = messages.len(),
                backend = "valkey",
                "bounded session recent messages loaded"
            );
            return Ok(messages);
        }

        let guard = self.inner.read().await;
        let Some(window) = guard.get(session_id) else {
            return Ok(Vec::new());
        };
        let turns = window.get_recent_turns(limit_slots);
        let messages = turns
            .iter()
            .map(|slot| ChatMessage {
                role: slot.role.clone(),
                content: Some(slot.content.clone()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            })
            .collect::<Vec<_>>();
        tracing::debug!(
            event = SessionEvent::BoundedRecentMessagesLoaded.as_str(),
            session_id,
            requested_turns = limit,
            loaded_messages = messages.len(),
            backend = "memory",
            "bounded session recent messages loaded"
        );
        Ok(messages)
    }

    /// Returns recent raw window slots (oldest to newest) for exact state snapshot/restore.
    ///
    /// # Errors
    /// Returns an error when loading bounded window slots from Valkey fails.
    pub async fn get_recent_slots(
        &self,
        session_id: &str,
        limit_slots: usize,
    ) -> Result<Vec<TurnSlot>> {
        if limit_slots == 0 {
            return Ok(Vec::new());
        }

        if let Some(ref redis) = self.redis {
            let slots = redis
                .get_recent_window_slots(session_id, limit_slots)
                .await
                .with_context(|| {
                    format!("valkey bounded session slot read failed for session_id={session_id}")
                })?;
            tracing::debug!(
                event = SessionEvent::SessionWindowSlotsLoaded.as_str(),
                session_id,
                requested_limit_slots = limit_slots,
                loaded_slots = slots.len(),
                backend = "valkey",
                "bounded session raw slots loaded"
            );
            return Ok(slots);
        }

        let guard = self.inner.read().await;
        let Some(window) = guard.get(session_id) else {
            return Ok(Vec::new());
        };
        let slots = window
            .get_recent_turns(limit_slots)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        tracing::debug!(
            event = SessionEvent::SessionWindowSlotsLoaded.as_str(),
            session_id,
            requested_limit_slots = limit_slots,
            loaded_slots = slots.len(),
            backend = "memory",
            "bounded session raw slots loaded"
        );
        Ok(slots)
    }

    /// Session stats: (`turn_count`, `total_tool_calls`, `ring_len`).
    ///
    /// # Errors
    /// Returns an error when reading bounded-session stats from Valkey fails.
    pub async fn get_stats(&self, session_id: &str) -> Result<Option<(u64, u64, usize)>> {
        if let Some(ref redis) = self.redis {
            let stats = redis.get_window_stats(session_id).await.with_context(|| {
                format!("valkey bounded session stats failed for session_id={session_id}")
            })?;
            let mapped = stats.map(|(slots, tool_calls, ring_len)| {
                let turn_count = slots / 2;
                (turn_count, tool_calls, ring_len)
            });
            if let Some((turn_count, tool_calls, ring_len)) = mapped {
                tracing::debug!(
                    event = SessionEvent::BoundedStatsLoaded.as_str(),
                    session_id,
                    turn_count,
                    tool_calls,
                    ring_len,
                    backend = "valkey",
                    "bounded session stats loaded"
                );
            }
            return Ok(mapped);
        }

        let guard = self.inner.read().await;
        let mapped = guard.get(session_id).map(|window| {
            let stats = window.get_stats();
            let turn_count = stats.total_turns / 2;
            (turn_count, stats.total_tool_calls, stats.window_used)
        });
        if let Some((turn_count, tool_calls, ring_len)) = mapped {
            tracing::debug!(
                event = SessionEvent::BoundedStatsLoaded.as_str(),
                session_id,
                turn_count,
                tool_calls,
                ring_len,
                backend = "memory",
                "bounded session stats loaded"
            );
        }
        Ok(mapped)
    }
}
