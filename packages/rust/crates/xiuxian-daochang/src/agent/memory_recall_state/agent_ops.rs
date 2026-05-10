use super::storage::{parse_snapshot_chat_message, snapshot_chat_message, snapshot_session_id};
use super::types::SessionMemoryRecallSnapshot;
use crate::agent::Agent;

impl Agent {
    pub(crate) async fn record_memory_recall_snapshot(
        &self,
        session_id: &str,
        snapshot: SessionMemoryRecallSnapshot,
    ) {
        let Some(message) = snapshot_chat_message(&snapshot) else {
            tracing::warn!(
                session_id,
                "failed to serialize memory recall snapshot payload"
            );
            return;
        };

        let storage_session_id = snapshot_session_id(session_id);
        if let Err(error) = self
            .session
            .replace(&storage_session_id, vec![message])
            .await
        {
            tracing::warn!(
                session_id,
                storage_session_id,
                error = %error,
                "failed to persist memory recall snapshot payload"
            );
            return;
        }
        if let Err(error) = self
            .session
            .publish_stream_event(
                self.memory_stream_name(),
                vec![
                    ("kind".to_string(), "recall_snapshot_updated".to_string()),
                    ("session_id".to_string(), session_id.to_string()),
                    ("storage_session_id".to_string(), storage_session_id.clone()),
                    (
                        "decision".to_string(),
                        snapshot.decision.as_str().to_string(),
                    ),
                    (
                        "recalled_selected".to_string(),
                        snapshot.recalled_selected.to_string(),
                    ),
                    (
                        "recalled_injected".to_string(),
                        snapshot.recalled_injected.to_string(),
                    ),
                    (
                        "pipeline_duration_ms".to_string(),
                        snapshot.pipeline_duration_ms.to_string(),
                    ),
                    (
                        "captured_at_unix_ms".to_string(),
                        snapshot.created_at_unix_ms.to_string(),
                    ),
                ],
            )
            .await
        {
            tracing::warn!(
                session_id,
                error = %error,
                "failed to publish memory recall snapshot stream event"
            );
        }
        tracing::debug!(
            session_id,
            storage_session_id,
            "memory recall snapshot persisted"
        );
    }

    /// Load the latest persisted memory-recall snapshot for the given session.
    ///
    /// Returns `None` when the session has no snapshot or deserialization fails.
    /// Load the latest persisted memory-recall snapshot for a session.
    ///
    /// Returns `None` when no snapshot exists or when persisted payloads cannot
    /// be loaded/parsing fails.
    pub async fn inspect_memory_recall_snapshot(
        &self,
        session_id: impl AsRef<str>,
    ) -> Option<SessionMemoryRecallSnapshot> {
        let session_id = session_id.as_ref();
        let storage_session_id = snapshot_session_id(session_id);
        let messages = match self.session.get(&storage_session_id).await {
            Ok(messages) => messages,
            Err(error) => {
                tracing::warn!(
                    session_id,
                    storage_session_id,
                    error = %error,
                    "failed to load memory recall snapshot payload"
                );
                return None;
            }
        };

        let snapshot = messages.iter().rev().find_map(parse_snapshot_chat_message);
        if snapshot.is_none() && !messages.is_empty() {
            tracing::warn!(
                session_id,
                storage_session_id,
                persisted_messages = messages.len(),
                "failed to parse persisted memory recall snapshot payload"
            );
        }
        snapshot
    }
}
