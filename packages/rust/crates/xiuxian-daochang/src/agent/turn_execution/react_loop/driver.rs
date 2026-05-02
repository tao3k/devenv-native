use anyhow::Result;

use crate::agent::Agent;

use super::types::{ReactConversationState, ReactPreparedMessages, TurnRuntimeContext};

impl Agent {
    pub(in crate::agent) async fn run_react_loop(
        &self,
        session_id: &str,
        user_message: &str,
        force_react: bool,
        turn_id: u64,
    ) -> Result<String> {
        let (decision, policy_hint) = self.prepare_react_decision(session_id, force_react).await;
        let ReactPreparedMessages {
            mut messages,
            summary_segment_count,
        } = self
            .prepare_react_messages(session_id, user_message, &decision, policy_hint.as_ref())
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
}
