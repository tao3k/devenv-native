//! Stdio gateway: read line from stdin → run agent turn → print output.

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::agent::Agent;

/// Default session ID when not overridden by flag.
pub const DEFAULT_STDIO_SESSION_ID: &str = "default";

/// Session identifier surface for the stdio gateway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdioSessionId(String);

impl StdioSessionId {
    /// Build a stdio session id.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for StdioSessionId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for StdioSessionId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Run stdio loop: read lines, run turn, print output. Exits on EOF or Ctrl+C.
///
/// * `agent` — the agent instance
/// * `session_id` — session ID for the conversation (e.g. from `--session-id`)
///
/// # Errors
/// Returns an error when stdin reads fail or agent turn execution fails.
pub async fn run_stdio(agent: Agent, session_id: StdioSessionId) -> Result<()> {
    let mut reader = BufReader::new(tokio::io::stdin()).lines();
    while let Some(line) = reader.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let out = agent.run_turn(session_id.as_str(), line).await?;
        println!("{out}");
    }
    Ok(())
}
