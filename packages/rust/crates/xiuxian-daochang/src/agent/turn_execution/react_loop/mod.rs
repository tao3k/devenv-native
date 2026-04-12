mod context_repair;
mod conversation;
mod memory_recall;
mod messages;
mod types;

use anyhow::Result;

use crate::agent::{Agent, omega};
use crate::contracts::{OmegaDecision, OmegaFallbackPolicy};
use crate::observability::SessionEvent;

use self::types::{ReactConversationState, ReactPreparedMessages, TurnRuntimeContext};

impl Agent {
    pub(in crate::agent) async fn run_react_loop(
        &self,
        session_id: &str,
        user_message: &str,
        force_react: bool,
        turn_id: u64,
    ) -> Result<String> {
        let decision = self.prepare_react_decision(session_id, force_react).await;
        let ReactPreparedMessages {
            mut messages,
            summary_segment_count,
        } = self
            .prepare_react_messages(session_id, user_message)
            .await?;
        let recall_credit_candidates = self
            .apply_memory_recall_if_enabled(
                session_id,
                user_message,
                &mut messages,
                summary_segment_count,
            )
            .await;
        let messages = self
            .normalize_and_pack_react_messages(session_id, turn_id, messages)
            .await;
        let tools_json = self.load_tools_json_for_react().await?;
        let mut state = ReactConversationState::new(messages, tools_json);
        let turn_ctx = TurnRuntimeContext {
            session_id,
            user_message,
            turn_id,
            route: decision.route,
            recall_credit_candidates: &recall_credit_candidates,
        };

        self.execute_react_rounds(&turn_ctx, &mut state).await
    }

    /// Streaming variant of [`Self::run_react_loop`].
    ///
    /// Same pre-loop setup (message preparation, memory recall, tool loading,
    /// omega decision). The inner loop uses [`Self::execute_react_rounds_stream`]
    /// to emit events through `event_tx`.
    pub(in crate::agent) async fn run_react_loop_stream(
        &self,
        session_id: &str,
        user_message: &str,
        force_react: bool,
        turn_id: u64,
        event_tx: &tokio::sync::mpsc::Sender<crate::agent::AgentStreamEvent>,
    ) -> Result<String> {
        let decision = self.prepare_react_decision(session_id, force_react).await;
        let ReactPreparedMessages {
            mut messages,
            summary_segment_count,
        } = self
            .prepare_react_messages(session_id, user_message)
            .await?;
        let recall_credit_candidates = self
            .apply_memory_recall_if_enabled(
                session_id,
                user_message,
                &mut messages,
                summary_segment_count,
            )
            .await;
        let messages = self
            .normalize_and_pack_react_messages(session_id, turn_id, messages)
            .await;
        let tools_json = self.load_tools_json_for_react().await?;
        let mut state = ReactConversationState::new(messages, tools_json);
        let turn_ctx = TurnRuntimeContext {
            session_id,
            user_message,
            turn_id,
            route: decision.route,
            recall_credit_candidates: &recall_credit_candidates,
        };

        Box::pin(self.execute_react_rounds_stream(&turn_ctx, &mut state, event_tx))
            .await
    }

    pub(super) async fn prepare_react_decision(
        &self,
        session_id: &str,
        force_react: bool,
    ) -> OmegaDecision {
        let policy_hint = self.take_reflection_policy_hint(session_id).await;
        if let Some(hint) = policy_hint.as_ref() {
            tracing::debug!(
                event = SessionEvent::ReflectionPolicyHintApplied.as_str(),
                session_id,
                source_turn_id = hint.source_turn_id,
                preferred_route = hint.preferred_route.as_str(),
                risk_floor = hint.risk_floor.as_str(),
                fallback_override = hint.fallback_override.map(OmegaFallbackPolicy::as_str),
                tool_trust_class = hint.tool_trust_class.as_str(),
                reason = %hint.reason,
                "reflection policy hint applied to route decision"
            );
        }
        let decision = omega::apply_quality_gate(omega::apply_policy_hint(
            omega::decide_for_standard_turn(force_react),
            policy_hint.as_ref(),
        ));
        Self::record_omega_decision(session_id, &decision, None, None);
        decision
    }
}
