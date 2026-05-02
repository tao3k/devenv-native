use anyhow::Result;

use crate::agent::Agent;
use crate::parse_react_shortcut;

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
}
