use anyhow::Result;

use crate::agent::Agent;
use crate::agent::AgentStreamEvent;
use crate::session::ChatMessage;

use super::context_repair::is_context_window_exceeded_error;
use super::types::{ReactConversationState, TurnRuntimeContext};

impl Agent {
    pub(super) async fn execute_react_rounds(
        &self,
        turn_ctx: &TurnRuntimeContext<'_>,
        state: &mut ReactConversationState,
    ) -> Result<String> {
        loop {
            self.ensure_tool_round_limit(turn_ctx, state).await?;
            state.round = state.round.saturating_add(1);

            let response = self.request_assistant_message(turn_ctx, state).await?;
            if let Some(out) = self
                .handle_assistant_response(turn_ctx, state, response)
                .await?
            {
                return Ok(out);
            }
        }
    }

    async fn ensure_tool_round_limit(
        &self,
        turn_ctx: &TurnRuntimeContext<'_>,
        state: &ReactConversationState,
    ) -> Result<()> {
        if state.round < self.config.max_tool_rounds {
            return Ok(());
        }

        let hint = format!(
            "max_tool_rounds ({}) exceeded after {} rounds ({} tool calls). \
            Try again with a fresh message (rounds reset per message), or increase \
            OMNI_AGENT_MAX_TOOL_ROUNDS / telegram.max_tool_rounds. \
            Last tools: {:?}",
            self.config.max_tool_rounds,
            state.round,
            state.total_tool_calls_this_turn,
            state.last_tool_names
        );
        tracing::warn!("{}", hint);
        self.finalize_turn_error(turn_ctx, state, &hint).await;
        Err(anyhow::anyhow!("{hint}"))
    }

    async fn request_assistant_message(
        &self,
        turn_ctx: &TurnRuntimeContext<'_>,
        state: &mut ReactConversationState,
    ) -> Result<crate::llm::AssistantMessage> {
        match self
            .llm
            .chat(state.messages.clone(), state.tools_json.clone())
            .await
        {
            Ok(response) => Ok(response),
            Err(error) if is_context_window_exceeded_error(&error) => {
                let repair_result = self
                    .repair_context_window_and_retry(turn_ctx, state, error)
                    .await?;
                state.messages = repair_result.messages;
                state.tools_json = repair_result.tools_json;
                Ok(repair_result.response)
            }
            Err(error) => Err(error),
        }
    }

    async fn handle_assistant_response(
        &self,
        turn_ctx: &TurnRuntimeContext<'_>,
        state: &mut ReactConversationState,
        response: crate::llm::AssistantMessage,
    ) -> Result<Option<String>> {
        if let Some(tool_calls) = response.tool_calls {
            if tool_calls.is_empty() {
                return self
                    .finalize_turn_success(turn_ctx, state, response.content.unwrap_or_default())
                    .await
                    .map(Some);
            }
            self.process_tool_calls(turn_ctx, state, response.content, tool_calls)
                .await?;
            return Ok(None);
        }

        self.finalize_turn_success(turn_ctx, state, response.content.unwrap_or_default())
            .await
            .map(Some)
    }

    async fn process_tool_calls(
        &self,
        turn_ctx: &TurnRuntimeContext<'_>,
        state: &mut ReactConversationState,
        assistant_content: Option<String>,
        tool_calls: Vec<crate::session::ToolCallOut>,
    ) -> Result<()> {
        state.total_tool_calls_this_turn = state
            .total_tool_calls_this_turn
            .saturating_add(u32::try_from(tool_calls.len()).unwrap_or(u32::MAX));
        state.last_tool_names = tool_calls
            .iter()
            .map(|tool_call| tool_call.function.name.clone())
            .collect();
        state.messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: assistant_content,
            tool_calls: Some(tool_calls.clone()),
            tool_call_id: None,
            name: None,
        });

        for tool_call in tool_calls {
            let name = tool_call.function.name.clone();
            let args = parse_tool_call_arguments(&tool_call.function.arguments);
            let output = match self
                .call_tool_with_diagnostics(
                    Some(turn_ctx.session_id),
                    Some(tool_call.id.as_str()),
                    &name,
                    args,
                )
                .await
            {
                Ok(output) => {
                    state.tool_summary.record_result(output.is_error);
                    output
                }
                Err(error) => {
                    if let Some(soft_output) = Self::soft_fail_tool_error_output(
                        &name,
                        Some(tool_call.id.as_str()),
                        &error,
                    ) {
                        state.tool_summary.record_result(true);
                        soft_output
                    } else {
                        state.tool_summary.record_transport_failure();
                        let error_text = format!("tool `{name}` call failed: {error}");
                        self.finalize_turn_error(turn_ctx, state, &error_text).await;
                        return Err(error);
                    }
                }
            };
            state.messages.push(ChatMessage {
                role: "tool".to_string(),
                content: Some(output.text),
                tool_calls: None,
                tool_call_id: output.tool_call_id.or_else(|| Some(tool_call.id.clone())),
                name: Some(name),
            });
        }

        Ok(())
    }

    async fn finalize_turn_success(
        &self,
        turn_ctx: &TurnRuntimeContext<'_>,
        state: &mut ReactConversationState,
        output: String,
    ) -> Result<String> {
        let outcome = self
            .update_recall_feedback(
                turn_ctx.session_id,
                turn_ctx.user_message,
                &output,
                Some(&state.tool_summary),
            )
            .await;
        self.apply_memory_recall_credit(
            turn_ctx.session_id,
            turn_ctx.recall_credit_candidates,
            outcome,
        );
        self.append_turn_to_session(
            turn_ctx.session_id,
            turn_ctx.user_message,
            &output,
            state.total_tool_calls_this_turn,
        )
        .await?;
        self.reflect_turn_and_update_policy_hint(
            turn_ctx.session_id,
            turn_ctx.turn_id,
            turn_ctx.route,
            turn_ctx.user_message,
            &output,
            "completed",
            state.total_tool_calls_this_turn,
        )
        .await;
        Ok(output)
    }

    async fn finalize_turn_error(
        &self,
        turn_ctx: &TurnRuntimeContext<'_>,
        state: &ReactConversationState,
        error_text: &str,
    ) {
        let outcome = self
            .update_recall_feedback(
                turn_ctx.session_id,
                turn_ctx.user_message,
                error_text,
                Some(&state.tool_summary),
            )
            .await;
        self.apply_memory_recall_credit(
            turn_ctx.session_id,
            turn_ctx.recall_credit_candidates,
            outcome,
        );
        self.reflect_turn_and_update_policy_hint(
            turn_ctx.session_id,
            turn_ctx.turn_id,
            turn_ctx.route,
            turn_ctx.user_message,
            error_text,
            "error",
            state.total_tool_calls_this_turn,
        )
        .await;
    }

    // ── Streaming variants ──────────────────────────────────────────

    /// Streaming variant of [`Self::execute_react_rounds`].
    ///
    /// Emits [`AgentStreamEvent`] values through `event_tx` as the LLM
    /// streams text and tool calls are executed.
    pub(in crate::agent) async fn execute_react_rounds_stream(
        &self,
        turn_ctx: &TurnRuntimeContext<'_>,
        state: &mut ReactConversationState,
        event_tx: &tokio::sync::mpsc::Sender<AgentStreamEvent>,
    ) -> Result<String> {
        loop {
            self.ensure_tool_round_limit(turn_ctx, state).await?;
            state.round = state.round.saturating_add(1);

            // Stream the LLM call -- text deltas are emitted via event_tx,
            // tool calls are accumulated and returned in the AssistantMessage.
            let response = self
                .request_assistant_message_stream(turn_ctx, state, event_tx)
                .await?;

            if let Some(tool_calls) = response.tool_calls {
                if tool_calls.is_empty() {
                    return self
                        .finalize_turn_success(
                            turn_ctx,
                            state,
                            response.content.unwrap_or_default(),
                        )
                        .await;
                }
                // Emit tool call events
                let _ = event_tx
                    .send(AgentStreamEvent::ToolCallsStarted {
                        tool_calls: tool_calls.clone(),
                    })
                    .await;

                // Process tool calls (reuse existing dispatch logic)
                self.process_tool_calls_stream(
                    turn_ctx,
                    state,
                    response.content,
                    tool_calls,
                    event_tx,
                )
                .await?;
                continue;
            }

            return self
                .finalize_turn_success(turn_ctx, state, response.content.unwrap_or_default())
                .await;
        }
    }

    async fn request_assistant_message_stream(
        &self,
        turn_ctx: &TurnRuntimeContext<'_>,
        state: &mut ReactConversationState,
        event_tx: &tokio::sync::mpsc::Sender<AgentStreamEvent>,
    ) -> Result<crate::llm::AssistantMessage> {
        match self
            .llm
            .chat_stream(
                state.messages.clone(),
                state.tools_json.clone(),
                event_tx,
            )
            .await
        {
            Ok(response) => Ok(response),
            Err(error) if is_context_window_exceeded_error(&error) => {
                // Context repair falls back to non-streaming retry.
                let repair_result = self
                    .repair_context_window_and_retry(turn_ctx, state, error)
                    .await?;
                state.messages = repair_result.messages;
                state.tools_json = repair_result.tools_json;
                // Emit the full repaired response as a single text delta.
                if let Some(ref content) = repair_result.response.content {
                    let _ = event_tx
                        .send(AgentStreamEvent::TextDelta(content.clone()))
                        .await;
                }
                Ok(repair_result.response)
            }
            Err(error) => Err(error),
        }
    }

    /// Streaming variant of [`Self::process_tool_calls`].
    ///
    /// Emits [`AgentStreamEvent::ToolResult`] after each tool execution.
    async fn process_tool_calls_stream(
        &self,
        turn_ctx: &TurnRuntimeContext<'_>,
        state: &mut ReactConversationState,
        assistant_content: Option<String>,
        tool_calls: Vec<crate::session::ToolCallOut>,
        event_tx: &tokio::sync::mpsc::Sender<AgentStreamEvent>,
    ) -> Result<()> {
        state.total_tool_calls_this_turn = state
            .total_tool_calls_this_turn
            .saturating_add(u32::try_from(tool_calls.len()).unwrap_or(u32::MAX));
        state.last_tool_names = tool_calls
            .iter()
            .map(|tool_call| tool_call.function.name.clone())
            .collect();
        state.messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: assistant_content,
            tool_calls: Some(tool_calls.clone()),
            tool_call_id: None,
            name: None,
        });

        for tool_call in tool_calls {
            let name = tool_call.function.name.clone();
            let args = parse_tool_call_arguments(&tool_call.function.arguments);
            let output = match self
                .call_tool_with_diagnostics(
                    Some(turn_ctx.session_id),
                    Some(tool_call.id.as_str()),
                    &name,
                    args,
                )
                .await
            {
                Ok(output) => {
                    state.tool_summary.record_result(output.is_error);
                    output
                }
                Err(error) => {
                    if let Some(soft_output) = Self::soft_fail_tool_error_output(
                        &name,
                        Some(tool_call.id.as_str()),
                        &error,
                    ) {
                        state.tool_summary.record_result(true);
                        soft_output
                    } else {
                        state.tool_summary.record_transport_failure();
                        let error_text = format!("tool `{name}` call failed: {error}");
                        self.finalize_turn_error(turn_ctx, state, &error_text).await;
                        return Err(error);
                    }
                }
            };

            // Emit tool result event
            let _ = event_tx
                .send(AgentStreamEvent::ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    name: name.clone(),
                    output: output.text.clone(),
                    is_error: output.is_error,
                })
                .await;

            state.messages.push(ChatMessage {
                role: "tool".to_string(),
                content: Some(output.text),
                tool_calls: None,
                tool_call_id: output.tool_call_id.or_else(|| Some(tool_call.id.clone())),
                name: Some(name),
            });
        }

        Ok(())
    }
}

fn parse_tool_call_arguments(arguments: &str) -> Option<serde_json::Value> {
    if arguments.is_empty() {
        None
    } else {
        serde_json::from_str(arguments).ok()
    }
}
