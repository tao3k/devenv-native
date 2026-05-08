use anyhow::{Context, Result};
use xiuxian_window::{SessionWindow, SessionWindowCheckpointId, TurnSlot};

use crate::observability::SessionEvent;
use crate::session::BoundedSessionStore;

impl BoundedSessionStore {
    /// Append one user/assistant turn pair into the bounded session window.
    ///
    /// # Errors
    /// Returns an error when Valkey-backed persistence fails.
    pub async fn append_turn(
        &self,
        session_id: &str,
        user_msg: &str,
        assistant_msg: &str,
        tool_count: u32,
    ) -> Result<()> {
        let slots = vec![
            TurnSlot::new("user", user_msg, 0),
            TurnSlot::new("assistant", assistant_msg, tool_count),
        ];

        if let Some(redis) = &self.redis {
            redis
                .append_window_slots(session_id, self.max_slots, &slots)
                .await
                .with_context(|| {
                    format!("valkey bounded session append failed for session_id={session_id}")
                })?;
        }

        let mut guard = self.inner.write().await;
        let window = guard
            .entry(session_id.to_string())
            .or_insert_with(|| SessionWindow::new(session_id, self.max_slots));
        window.append_turn("user", user_msg, 0, None);
        window.append_turn("assistant", assistant_msg, tool_count, None);

        tracing::debug!(
            event = SessionEvent::SessionWindowSlotsAppended.as_str(),
            session_id,
            appended_slots = slots.len(),
            tool_count,
            backend = if self.redis.is_some() {
                "valkey+memory"
            } else {
                "memory"
            },
            "bounded session turn appended"
        );
        Ok(())
    }

    /// Drain the oldest turns from the bounded session window.
    ///
    /// # Errors
    /// Returns an error when Valkey-backed persistence fails.
    pub async fn drain_oldest_turns(
        &self,
        session_id: &str,
        turns: usize,
    ) -> Result<Vec<(String, String, u32)>> {
        if turns == 0 {
            return Ok(Vec::new());
        }
        let slot_limit = turns.saturating_mul(2);

        let drained_slots = if let Some(redis) = &self.redis {
            redis
                .drain_oldest_window_slots(session_id, slot_limit)
                .await
                .with_context(|| {
                    format!("valkey bounded session drain failed for session_id={session_id}")
                })?
        } else {
            let mut guard = self.inner.write().await;
            let Some(window) = guard.get_mut(session_id) else {
                return Ok(Vec::new());
            };
            let drained = window.drain_oldest_turns(slot_limit);
            if window.get_stats().window_used == 0 {
                guard.remove(session_id);
            }
            drained
        };

        tracing::debug!(
            event = SessionEvent::SessionWindowSlotsDrained.as_str(),
            session_id,
            requested_turns = turns,
            requested_slots = slot_limit,
            drained_slots = drained_slots.len(),
            "bounded session oldest turns drained"
        );

        Ok(drained_slots_to_tuples(drained_slots))
    }

    /// Replace the full bounded session window with the provided raw slots.
    ///
    /// # Errors
    /// Returns an error when Valkey-backed persistence fails.
    pub async fn replace_window_slots(&self, session_id: &str, slots: &[TurnSlot]) -> Result<()> {
        if let Some(redis) = &self.redis {
            redis.clear_window(session_id).await.with_context(|| {
                format!("valkey bounded session clear failed for session_id={session_id}")
            })?;
            if !slots.is_empty() {
                redis
                    .append_window_slots(session_id, self.max_slots, slots)
                    .await
                    .with_context(|| {
                        format!("valkey bounded session replace failed for session_id={session_id}")
                    })?;
            }
        }

        let mut guard = self.inner.write().await;
        if slots.is_empty() {
            guard.remove(session_id);
        } else {
            let mut window = SessionWindow::new(session_id, self.max_slots);
            for slot in slots {
                window.append_turn(
                    &slot.role,
                    &slot.content,
                    slot.tool_count,
                    slot.checkpoint_id
                        .as_deref()
                        .map(SessionWindowCheckpointId::from),
                );
            }
            guard.insert(session_id.to_string(), window);
        }

        tracing::debug!(
            event = SessionEvent::SessionWindowSlotsLoaded.as_str(),
            session_id,
            replaced_slots = slots.len(),
            "bounded session window slots replaced"
        );
        Ok(())
    }

    /// Clear bounded window and summary state for one session.
    ///
    /// # Errors
    /// Returns an error when Valkey-backed persistence fails.
    pub async fn clear(&self, session_id: &str) -> Result<()> {
        if let Some(redis) = &self.redis {
            redis.clear_window(session_id).await.with_context(|| {
                format!("valkey bounded session window clear failed for session_id={session_id}")
            })?;
            redis.clear_summary(session_id).await.with_context(|| {
                format!("valkey bounded session summary clear failed for session_id={session_id}")
            })?;
        }

        self.inner.write().await.remove(session_id);
        self.summaries.write().await.remove(session_id);

        tracing::debug!(
            event = SessionEvent::SessionWindowCleared.as_str(),
            session_id,
            "bounded session window cleared"
        );
        tracing::debug!(
            event = SessionEvent::SessionSummaryCleared.as_str(),
            session_id,
            "bounded session summary cleared"
        );
        Ok(())
    }
}

fn drained_slots_to_tuples(slots: Vec<TurnSlot>) -> Vec<(String, String, u32)> {
    slots
        .into_iter()
        .map(|slot| (slot.role, slot.content, slot.tool_count))
        .collect()
}
