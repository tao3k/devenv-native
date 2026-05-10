use anyhow::Result;

use crate::agent::Agent;

impl Agent {
    #[doc(hidden)]
    /// # Errors
    /// Returns an error when appending the turn into session storage fails.
    pub async fn append_turn_for_session(
        &self,
        session_id: impl AsRef<str>,
        user_msg: &str,
        assistant_msg: &str,
    ) -> Result<()> {
        let session_id = session_id.as_ref();
        self.append_turn_to_session(session_id, user_msg, assistant_msg, 0)
            .await
    }

    /// # Errors
    /// Returns an error when appending the turn into session storage fails.
    pub async fn append_turn_with_tool_count_for_session(
        &self,
        session_id: impl AsRef<str>,
        user_msg: &str,
        assistant_msg: &str,
        tool_count: u32,
    ) -> Result<()> {
        let session_id = session_id.as_ref();
        self.append_turn_to_session(session_id, user_msg, assistant_msg, tool_count)
            .await
    }
}
