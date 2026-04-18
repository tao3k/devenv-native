use anyhow::Result;
use xiuxian_qianhuan::InjectionPolicy;

use crate::agent::reflection::PolicyHintDirective;
use crate::agent::system_prompt_injection_state::SYSTEM_PROMPT_INJECTION_CONTEXT_MESSAGE_NAME;
use crate::agent::{Agent, context_budget, injection};
use crate::contracts::OmegaDecision;
use crate::session::{ChatMessage, SessionSummarySegment};

use super::types::ReactPreparedMessages;

const NEXT_TURN_HINT_MESSAGE_NAME: &str = "agent.next_turn_hint";

impl Agent {
    pub(super) async fn prepare_react_messages(
        &self,
        session_id: &str,
        user_message: &str,
        decision: &OmegaDecision,
        policy_hint: Option<&PolicyHintDirective>,
    ) -> Result<ReactPreparedMessages> {
        let mut summary_segments: Vec<SessionSummarySegment> = Vec::new();
        let mut messages: Vec<ChatMessage> = if let Some(ref w) = self.bounded_session {
            let limit = self.config.window_max_turns.unwrap_or(512);
            summary_segments = w
                .get_recent_summary_segments(session_id, self.config.summary_max_segments)
                .await?;
            w.get_recent_messages(session_id, limit).await?
        } else {
            self.session.get(session_id).await?
        };

        Self::prepend_summary_segments(&mut messages, &summary_segments);

        if let Some(snapshot) = self
            .inspect_session_system_prompt_injection(session_id)
            .await
        {
            messages.insert(
                0,
                ChatMessage {
                    role: "system".to_string(),
                    content: Some(render_system_prompt_injection_context(&snapshot.xml)),
                    tool_calls: None,
                    tool_call_id: None,
                    name: Some(SYSTEM_PROMPT_INJECTION_CONTEXT_MESSAGE_NAME.to_string()),
                },
            );
        }

        if let Some(hint) = policy_hint {
            messages.insert(
                0,
                ChatMessage {
                    role: "system".to_string(),
                    content: Some(render_next_turn_hint(hint, decision)),
                    tool_calls: None,
                    tool_call_id: None,
                    name: Some(NEXT_TURN_HINT_MESSAGE_NAME.to_string()),
                },
            );
        }

        messages.push(ChatMessage {
            role: "user".to_string(),
            content: Some(user_message.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });

        Ok(ReactPreparedMessages {
            messages,
            summary_segment_count: summary_segments.len(),
        })
    }

    pub(super) fn prepend_summary_segments(
        messages: &mut Vec<ChatMessage>,
        summary_segments: &[SessionSummarySegment],
    ) {
        if summary_segments.is_empty() {
            return;
        }
        let segment_count = summary_segments.len();
        let summary_messages = summary_segments
            .iter()
            .enumerate()
            .map(|(index, segment)| ChatMessage {
                role: "system".to_string(),
                content: Some(format!(
                    "Compressed conversation history from older turns (segment {}/{}): {} (turns={}, tools={})",
                    index + 1,
                    segment_count,
                    segment.summary,
                    segment.turn_count,
                    segment.tool_calls
                )),
                tool_calls: None,
                tool_call_id: None,
                name: Some(context_budget::SESSION_SUMMARY_MESSAGE_NAME.to_string()),
            })
            .collect::<Vec<_>>();
        messages.splice(0..0, summary_messages);
    }

    pub(super) async fn normalize_and_pack_react_messages(
        &self,
        session_id: &str,
        turn_id: u64,
        messages: Vec<ChatMessage>,
    ) -> Vec<ChatMessage> {
        let raw_messages = messages;
        let mut messages = match injection::normalize_messages_with_snapshot(
            session_id,
            turn_id,
            raw_messages.clone(),
            InjectionPolicy::default(),
        ) {
            Ok(normalized) => {
                if let Some(snapshot) = normalized.snapshot.as_ref() {
                    Self::record_injection_snapshot(session_id, snapshot);
                }
                normalized.messages
            }
            Err(error) => {
                tracing::warn!(
                    session_id,
                    error = %error,
                    "failed to normalize injection snapshot; context messages unchanged"
                );
                raw_messages
            }
        };

        if let Some(context_budget_tokens) = self.config.context_budget_tokens
            && context_budget_tokens > 0
        {
            let result = context_budget::prune_messages_for_token_budget_with_strategy(
                messages,
                context_budget_tokens,
                self.config.context_budget_reserve_tokens,
                self.config.context_budget_strategy,
            );
            messages = result.messages;
            let report = result.report;
            self.record_context_budget_snapshot(session_id, &report)
                .await;
            tracing::debug!(
                session_id,
                strategy = report.strategy.as_str(),
                budget_tokens = report.budget_tokens,
                reserve_tokens = report.reserve_tokens,
                effective_budget_tokens = report.effective_budget_tokens,
                pre_messages = report.pre_messages,
                post_messages = report.post_messages,
                pre_tokens = report.pre_tokens,
                post_tokens = report.post_tokens,
                dropped_messages = report.pre_messages.saturating_sub(report.post_messages),
                dropped_tokens = report.pre_tokens.saturating_sub(report.post_tokens),
                non_system_pre_messages = report.non_system.input_messages,
                non_system_kept_messages = report.non_system.kept_messages,
                non_system_dropped_messages = report.non_system.dropped_messages(),
                non_system_pre_tokens = report.non_system.input_tokens,
                non_system_kept_tokens = report.non_system.kept_tokens,
                non_system_dropped_tokens = report.non_system.dropped_tokens(),
                non_system_truncated_messages = report.non_system.truncated_messages,
                non_system_truncated_tokens = report.non_system.truncated_tokens,
                regular_system_pre_messages = report.regular_system.input_messages,
                regular_system_kept_messages = report.regular_system.kept_messages,
                regular_system_dropped_messages = report.regular_system.dropped_messages(),
                regular_system_pre_tokens = report.regular_system.input_tokens,
                regular_system_kept_tokens = report.regular_system.kept_tokens,
                regular_system_dropped_tokens = report.regular_system.dropped_tokens(),
                regular_system_truncated_messages = report.regular_system.truncated_messages,
                regular_system_truncated_tokens = report.regular_system.truncated_tokens,
                summary_pre_messages = report.summary_system.input_messages,
                summary_kept_messages = report.summary_system.kept_messages,
                summary_dropped_messages = report.summary_system.dropped_messages(),
                summary_pre_tokens = report.summary_system.input_tokens,
                summary_kept_tokens = report.summary_system.kept_tokens,
                summary_dropped_tokens = report.summary_system.dropped_tokens(),
                summary_truncated_messages = report.summary_system.truncated_messages,
                summary_truncated_tokens = report.summary_system.truncated_tokens,
                "applied token-budget context packing"
            );
        }

        messages
    }

    pub(super) async fn load_tools_json_for_react(&self) -> Result<Option<Vec<serde_json::Value>>> {
        self.tool_definitions_for_llm().await
    }
}

fn render_next_turn_hint(hint: &PolicyHintDirective, decision: &OmegaDecision) -> String {
    let role_mix_profile = match decision.policy_id.as_deref() {
        Some(policy_id) if policy_id.contains(".recovery.") => "recovery",
        _ => "normal",
    };
    let fallback_override = hint
        .fallback_override
        .map_or("none", crate::contracts::OmegaFallbackPolicy::as_str);

    format!(
        "source_turn_id={}; reason={}; preferred_route={}; risk_floor={}; fallback_override={}; tool_trust_class={}; role_mix_profile={}",
        hint.source_turn_id,
        hint.reason,
        hint.preferred_route.as_str(),
        hint.risk_floor.as_str(),
        fallback_override,
        hint.tool_trust_class.as_str(),
        role_mix_profile,
    )
}

fn render_system_prompt_injection_context(xml: &str) -> String {
    format!(
        "Session-scoped system prompt injection context. Preserve these anchoring instructions under budget pressure, keep genesis_rules and persona_steering guidance available during planning, and treat the XML below as authoritative session steering.\n{xml}",
    )
}
