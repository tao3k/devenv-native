mod react_loop;
/// Typed events for streaming agent turn output.
pub mod stream_events;

use anyhow::Result;

pub use stream_events::AgentStreamEvent;

use crate::agent::Agent;
use crate::shortcuts::parse_react_shortcut;

impl Agent {
    /// Execute one user turn using the `ReAct` loop.
    ///
    /// # Errors
    /// Returns an error when the `ReAct` loop execution fails.
    pub async fn run_turn(&self, session_id: &str, user_message: &str) -> Result<String> {
        self.enforce_session_reset_policy(session_id).await?;
        let forced_react_message = parse_react_shortcut(user_message);
        let force_react = forced_react_message.is_some();
        let user_message_owned = forced_react_message.unwrap_or_else(|| user_message.to_string());
        let turn_id = Self::next_runtime_turn_id();

        // System shortcuts like !react are handled, but external tool shortcuts
        // have been removed in favor of pure ReAct tool calls.

        Box::pin(self.run_react_loop(session_id, &user_message_owned, force_react, turn_id)).await
    }

    /// Streaming variant of [`Self::run_turn`].
    ///
    /// Emits [`AgentStreamEvent`] values through `event_tx` as the LLM
    /// generates text and tools are executed. Returns the final output
    /// string (same as `run_turn`).
    ///
    /// The caller should read from the corresponding
    /// `mpsc::Receiver<AgentStreamEvent>` concurrently (e.g. in a separate
    /// task or by spawning this method in a task).
    ///
    /// On completion, a [`AgentStreamEvent::TurnComplete`] or
    /// [`AgentStreamEvent::TurnError`] is sent automatically before returning.
    ///
    /// # Errors
    /// Returns an error when the `ReAct` loop execution fails.
    pub async fn run_turn_stream(
        &self,
        session_id: &str,
        user_message: &str,
        event_tx: &tokio::sync::mpsc::Sender<AgentStreamEvent>,
    ) -> Result<String> {
        self.enforce_session_reset_policy(session_id).await?;
        let forced_react_message = parse_react_shortcut(user_message);
        let force_react = forced_react_message.is_some();
        let user_message_owned = forced_react_message.unwrap_or_else(|| user_message.to_string());
        let turn_id = Self::next_runtime_turn_id();

        let result = Box::pin(self.run_react_loop_stream(
            session_id,
            &user_message_owned,
            force_react,
            turn_id,
            event_tx,
        ))
        .await;

        match &result {
            Ok(output) => {
                let _ = event_tx
                    .send(AgentStreamEvent::TurnComplete {
                        final_output: output.clone(),
                    })
                    .await;
            }
            Err(e) => {
                let _ = event_tx
                    .send(AgentStreamEvent::TurnError {
                        error: e.to_string(),
                    })
                    .await;
            }
        }

        result
    }
}
