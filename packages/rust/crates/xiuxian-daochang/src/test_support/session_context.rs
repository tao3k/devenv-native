//! Session-context helpers exposed for integration tests.

use crate::agent::session_context;
use crate::{Agent, AgentConfig, ChatMessage, SessionSummarySegment};

/// Build a test agent with optional bounded context window size.
///
/// # Errors
///
/// Returns an error when agent bootstrap fails.
pub async fn build_session_context_test_agent(
    window_max_turns: Option<usize>,
) -> anyhow::Result<Agent> {
    let config = AgentConfig {
        inference_url: "http://127.0.0.1:4000/v1/chat/completions".to_string(),
        window_max_turns,
        memory: None,
        consolidation_threshold_turns: None,
        ..AgentConfig::default()
    };
    Agent::from_config(config).await
}

#[must_use]
/// Returns the current Unix timestamp in milliseconds.
pub fn now_unix_ms() -> u64 {
    session_context::test_now_unix_ms()
}

/// Overrides the session reset idle timeout on a test agent.
pub fn set_session_reset_idle_timeout_ms(agent: &mut Agent, timeout_ms: Option<u64>) {
    agent.test_set_session_reset_idle_timeout_ms(timeout_ms);
}

/// Sets the last-activity timestamp for a test session.
pub async fn set_session_last_activity(agent: &Agent, session_id: &str, unix_ms: u64) {
    agent
        .test_set_session_last_activity(session_id, unix_ms)
        .await;
}

/// Apply idle session reset policy.
///
/// # Errors
///
/// Returns an error when reset policy or storage operations fail.
pub async fn enforce_session_reset_policy(agent: &Agent, session_id: &str) -> anyhow::Result<()> {
    agent.test_enforce_session_reset_policy(session_id).await
}

/// Load unbounded session messages.
///
/// # Errors
///
/// Returns an error when backend read fails.
pub async fn session_messages(agent: &Agent, session_id: &str) -> anyhow::Result<Vec<ChatMessage>> {
    agent.test_session_messages(session_id).await
}

/// Load bounded recent messages.
///
/// # Errors
///
/// Returns an error when bounded backend read fails.
pub async fn bounded_recent_messages(
    agent: &Agent,
    session_id: &str,
    limit: usize,
) -> anyhow::Result<Vec<ChatMessage>> {
    agent.test_bounded_recent_messages(session_id, limit).await
}

/// Load bounded recent summary segments.
///
/// # Errors
///
/// Returns an error when bounded backend read fails.
pub async fn bounded_recent_summary_segments(
    agent: &Agent,
    session_id: &str,
    limit: usize,
) -> anyhow::Result<Vec<SessionSummarySegment>> {
    agent
        .test_bounded_recent_summary_segments(session_id, limit)
        .await
}
