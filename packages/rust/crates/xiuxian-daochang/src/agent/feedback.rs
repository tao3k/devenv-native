use anyhow::Result;
use num_traits::ToPrimitive;

use crate::agent::memory::{RecalledEpisodeCandidate, apply_recall_credit};
use crate::agent::memory_recall_feedback::{
    RECALL_FEEDBACK_SOURCE_COMMAND, RecallOutcome, ToolExecutionSummary, resolve_feedback_outcome,
    update_feedback_bias,
};
use crate::agent::{Agent, SessionRecallFeedbackDirection, SessionRecallFeedbackUpdate};
use crate::observability::SessionEvent;

impl Agent {
    /// Clear session history for a session.
    ///
    /// # Errors
    ///
    /// Returns an error when bounded-session or session storage cleanup fails.
    pub async fn clear_session(&self, session_id: impl AsRef<str>) -> Result<()> {
        let session_id = session_id.as_ref();
        if let Some(ref w) = self.bounded_session {
            w.clear(session_id).await?;
        }
        self.memory_recall_feedback.write().await.remove(session_id);
        self.reflection_policy_hints
            .write()
            .await
            .remove(session_id);
        self.clear_memory_recall_feedback_bias(session_id);
        let _ = self.clear_session_system_prompt_injection(session_id).await;
        self.session.clear(session_id).await
    }

    /// Apply explicit recall feedback for a session.
    ///
    /// Returns `None` when memory is disabled.
    pub async fn apply_session_recall_feedback(
        &self,
        session_id: impl AsRef<str>,
        direction: SessionRecallFeedbackDirection,
    ) -> Option<SessionRecallFeedbackUpdate> {
        let session_id = session_id.as_ref();
        self.memory_store.as_ref()?;
        let outcome = match direction {
            SessionRecallFeedbackDirection::Up => RecallOutcome::Success,
            SessionRecallFeedbackDirection::Down => RecallOutcome::Failure,
        };
        let (previous, updated) = self
            .apply_recall_feedback_outcome(
                session_id,
                outcome,
                RECALL_FEEDBACK_SOURCE_COMMAND,
                None,
            )
            .await;
        Some(SessionRecallFeedbackUpdate {
            previous_bias: previous,
            updated_bias: updated,
            direction,
        })
    }

    pub(in crate::agent) async fn recall_feedback_bias(&self, session_id: &str) -> f32 {
        if let Some(bias) = self
            .memory_recall_feedback
            .read()
            .await
            .get(session_id)
            .copied()
        {
            return bias;
        }
        if let Some(bias) = self.load_memory_recall_feedback_bias(session_id) {
            self.memory_recall_feedback
                .write()
                .await
                .insert(session_id.to_string(), bias);
            return bias;
        }
        0.0
    }

    pub(in crate::agent) fn load_memory_recall_feedback_bias(
        &self,
        session_id: &str,
    ) -> Option<f32> {
        self.memory_store
            .as_ref()
            .and_then(|store| store.recall_feedback_bias(session_id))
    }

    pub(in crate::agent) fn persist_memory_recall_feedback_bias(
        &self,
        session_id: &str,
        bias: f32,
    ) {
        if let Some(store) = self.memory_store.as_ref() {
            store.set_recall_feedback_bias(session_id, bias);
        }
        self.persist_memory_recall_feedback_bias_atomic(session_id, bias, "feedback_update");
    }

    pub(in crate::agent) fn clear_memory_recall_feedback_bias(&self, session_id: &str) {
        if let Some(store) = self.memory_store.as_ref() {
            store.clear_recall_feedback_bias(session_id);
        }
        self.clear_memory_recall_feedback_bias_atomic(session_id, "feedback_clear");
    }

    pub(in crate::agent) async fn update_recall_feedback(
        &self,
        session_id: &str,
        user_message: &str,
        assistant_message: &str,
        tool_summary: Option<&ToolExecutionSummary>,
    ) -> Option<RecallOutcome> {
        self.memory_store.as_ref()?;
        let (outcome, source) =
            resolve_feedback_outcome(user_message, tool_summary, assistant_message);
        self.apply_recall_feedback_outcome(session_id, outcome, source, tool_summary)
            .await;
        Some(outcome)
    }

    pub(in crate::agent) fn apply_memory_recall_credit(
        &self,
        session_id: &str,
        candidates: &[RecalledEpisodeCandidate],
        outcome: Option<RecallOutcome>,
    ) {
        let Some(store) = self.memory_store.as_ref() else {
            return;
        };
        let Some(outcome) = outcome else {
            return;
        };
        if candidates.is_empty() {
            return;
        }
        let updates = apply_recall_credit(store, candidates, outcome);
        if updates.is_empty() {
            return;
        }
        let total_delta: f32 = updates.iter().map(|u| u.updated_q - u.previous_q).sum();
        let Some(applied_count) = updates.len().to_f64() else {
            return;
        };
        let avg_weight = updates
            .iter()
            .map(|update| f64::from(update.weight))
            .sum::<f64>()
            / applied_count;
        tracing::debug!(
            event = SessionEvent::MemoryRecallCreditApplied.as_str(),
            session_id,
            outcome = outcome.as_str(),
            candidates = candidates.len(),
            applied = updates.len(),
            avg_weight,
            total_q_delta = total_delta,
            "memory recall credit applied"
        );
    }

    pub(in crate::agent) async fn apply_recall_feedback_outcome(
        &self,
        session_id: &str,
        outcome: RecallOutcome,
        source: &str,
        tool_summary: Option<&ToolExecutionSummary>,
    ) -> (f32, f32) {
        let previous = self.recall_feedback_bias(session_id).await;
        let updated = update_feedback_bias(previous, outcome);
        self.memory_recall_feedback
            .write()
            .await
            .insert(session_id.to_string(), updated);
        self.persist_memory_recall_feedback_bias(session_id, updated);
        tracing::debug!(
            event = SessionEvent::MemoryRecallFeedbackUpdated.as_str(),
            session_id,
            outcome = outcome.as_str(),
            feedback_source = source,
            tool_attempted = tool_summary.map_or(0, |summary| summary.attempted),
            tool_succeeded = tool_summary.map_or(0, |summary| summary.succeeded),
            tool_failed = tool_summary.map_or(0, |summary| summary.failed),
            recall_feedback_bias_before = previous,
            recall_feedback_bias_after = updated,
            "memory recall feedback updated"
        );
        (previous, updated)
    }
}
