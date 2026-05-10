//! Top-level integration harness for `agent::session_context`.

pub(crate) mod observability {
    #[derive(Clone, Copy, Debug)]
    pub(crate) enum SessionEvent {
        BoundedStatsLoaded,
        ContextBackupCaptured,
        ContextWindowReset,
        ContextWindowResumeMissing,
        ContextWindowResumed,
        ContextWindowSnapshotDropped,
        ContextWindowSnapshotInspected,
        SessionMessagesLoaded,
    }

    impl SessionEvent {
        pub(crate) const fn as_str(self) -> &'static str {
            match self {
                Self::BoundedStatsLoaded => "bounded_stats_loaded",
                Self::ContextBackupCaptured => "context_backup_captured",
                Self::ContextWindowReset => "context_window_reset",
                Self::ContextWindowResumeMissing => "context_window_resume_missing",
                Self::ContextWindowResumed => "context_window_resumed",
                Self::ContextWindowSnapshotDropped => "context_window_snapshot_dropped",
                Self::ContextWindowSnapshotInspected => "context_window_snapshot_inspected",
                Self::SessionMessagesLoaded => "session_messages_loaded",
            }
        }
    }

    fn lint_symbol_probe() {
        let _ = (
            SessionEvent::BoundedStatsLoaded,
            SessionEvent::ContextBackupCaptured,
            SessionEvent::ContextWindowReset,
            SessionEvent::ContextWindowResumeMissing,
            SessionEvent::ContextWindowResumed,
            SessionEvent::ContextWindowSnapshotDropped,
            SessionEvent::ContextWindowSnapshotInspected,
            SessionEvent::SessionMessagesLoaded,
        );
    }

    const _: fn() = lint_symbol_probe;
}

pub(crate) mod agent {
    use std::collections::HashMap;
    use std::sync::Arc;

    use anyhow::Result;
    use tokio::sync::RwLock;

    pub(crate) use session_context::{SessionContextMode, SessionContextWindowInfo};
    use xiuxian_daochang::{AgentConfig, BoundedSessionStore, ChatMessage, SessionStore};

    pub(crate) struct Agent {
        pub(crate) config: AgentConfig,
        pub(crate) session: SessionStore,
        pub(crate) session_reset_idle_timeout_ms: Option<u64>,
        pub(crate) session_last_activity_unix_ms: Arc<RwLock<HashMap<String, u64>>>,
        pub(crate) bounded_session: Option<BoundedSessionStore>,
    }

    impl Agent {
        pub(crate) async fn from_config(config: AgentConfig) -> Result<Self> {
            std::future::ready(()).await;
            let session = SessionStore::new()?;
            let bounded_session = match config.window_max_turns {
                Some(max_turns) => Some(BoundedSessionStore::new_with_limits(
                    max_turns,
                    config.summary_max_segments,
                    config.summary_max_chars,
                )?),
                None => None,
            };
            Ok(Self {
                config,
                session,
                session_reset_idle_timeout_ms: None,
                session_last_activity_unix_ms: Arc::new(RwLock::new(HashMap::new())),
                bounded_session,
            })
        }

        pub(crate) async fn clear_session(&self, session_id: &str) -> Result<()> {
            if let Some(ref bounded_session) = self.bounded_session {
                bounded_session.clear(session_id).await?;
            }
            self.session.clear(session_id).await
        }

        async fn append_turn_to_session(
            &self,
            session_id: &str,
            user_msg: &str,
            assistant_msg: &str,
            tool_count: u32,
        ) -> Result<()> {
            if let Some(ref bounded_session) = self.bounded_session {
                bounded_session
                    .append_turn(session_id, user_msg, assistant_msg, tool_count)
                    .await?;
                return Ok(());
            }

            self.session
                .append(
                    session_id,
                    vec![
                        ChatMessage {
                            role: "user".to_string(),
                            content: Some(user_msg.to_string()),
                            tool_calls: None,
                            tool_call_id: None,
                            name: None,
                        },
                        ChatMessage {
                            role: "assistant".to_string(),
                            content: Some(assistant_msg.to_string()),
                            tool_calls: None,
                            tool_call_id: None,
                            name: None,
                        },
                    ],
                )
                .await
        }
    }

    #[path = "../../../../src/agent/session_context/mod.rs"]
    pub(crate) mod session_context;

    fn session_context_lint_symbol_probe() {
        let _ = crate::agent::Agent::from_config;
        let _ = session_context::now_unix_ms;
        let _ = session_context::test_now_unix_ms;
        let _ = crate::agent::Agent::test_set_session_reset_idle_timeout_ms;
        let _ = crate::agent::Agent::test_set_session_last_activity;
        let _ = crate::agent::Agent::test_enforce_session_reset_policy;
        let _ = crate::agent::Agent::test_session_messages;
        let _ = crate::agent::Agent::test_bounded_recent_messages;
        let _ = crate::agent::Agent::test_bounded_recent_summary_segments;
        let _ = std::mem::size_of::<session_context::SessionContextMode>();
        let _ = std::mem::size_of::<session_context::SessionContextSnapshotInfo>();
        let _ = std::mem::size_of::<session_context::SessionContextWindowInfo>();
    }

    const _: fn() = session_context_lint_symbol_probe;

    async fn session_context_api_exercise(agent: &Agent) -> Result<()> {
        let _ = agent
            .append_turn_to_session("session", "user", "assistant", 0)
            .await;
        let _ = agent
            .append_turn_with_tool_count_for_session("session", "user", "assistant", 0)
            .await;
        let _ = agent
            .append_turn_for_session("session", "user", "assistant")
            .await;
        let _ = agent.inspect_context_window("session").await;
        let _ = agent.reset_context_window("session").await;
        let _ = agent.peek_context_window_backup("session").await;
        let _ = agent.resume_context_window("session").await;
        let _ = agent.drop_context_window_backup("session").await;

        let _ = session_context::SessionContextMode::Bounded;
        let _ = session_context::SessionContextMode::Unbounded;
        Ok(())
    }

    fn session_context_api_exercise_probe() {
        let _ = session_context_api_exercise;
    }

    const _: fn() = session_context_api_exercise_probe;

    mod tests {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/unit/agent/session_context/window.rs"
        ));
    }
}
