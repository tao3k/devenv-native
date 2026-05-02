use xiuxian_llm::embedding::repair_embedding_dimension;
use xiuxian_memory_engine::{Episode, EpisodeStore};

use crate::agent::Agent;
use crate::agent::memory_recall_feedback::classify_assistant_outcome;
use crate::observability::SessionEvent;

use super::types::{StoredTurnEpisode, TurnStoreOutcome};

impl Agent {
    pub(super) async fn resolve_turn_episode(
        &self,
        store: &EpisodeStore,
        session_id: &str,
        user_msg: &str,
        assistant_msg: &str,
        tool_count: u32,
        outcome: &TurnStoreOutcome,
    ) -> Option<StoredTurnEpisode> {
        if let Some(existing_id) = find_existing_turn_episode_id(store, session_id, user_msg) {
            return Some(StoredTurnEpisode {
                id: existing_id,
                source: "existing",
            });
        }

        let id = build_turn_episode_id(session_id);
        let expected_dim = self
            .config
            .memory
            .as_ref()
            .map_or_else(|| store.encoder().dimension(), |cfg| cfg.embedding_dim);
        let embedding = self
            .build_turn_embedding_with_fallback(
                store,
                session_id,
                user_msg,
                tool_count,
                expected_dim,
            )
            .await;
        let episode = Episode::new(
            id.clone(),
            user_msg.to_string(),
            embedding,
            assistant_msg.to_string(),
            outcome.label.clone(),
        );
        if let Err(error) = store.store_for_scope(session_id, episode) {
            self.handle_turn_store_failure(session_id, &error).await;
            return None;
        }
        Some(StoredTurnEpisode { id, source: "new" })
    }

    async fn build_turn_embedding_with_fallback(
        &self,
        store: &EpisodeStore,
        session_id: &str,
        user_msg: &str,
        tool_count: u32,
        expected_dim: usize,
    ) -> Vec<f32> {
        match self.embedding_for_memory(user_msg, expected_dim).await {
            Ok(embedding) => embedding,
            Err(error_kind) => {
                let fallback_embedding = store.encoder().encode(user_msg);
                let fallback_dim = fallback_embedding.len();
                let repaired_fallback = if fallback_dim == expected_dim {
                    fallback_embedding
                } else {
                    repair_embedding_dimension(&fallback_embedding, expected_dim)
                };

                tracing::warn!(
                    event = SessionEvent::MemoryTurnStoreFailed.as_str(),
                    session_id,
                    reason = error_kind.as_str(),
                    tool_count,
                    fallback_strategy = "hash_encoder",
                    fallback_dim,
                    expected_dim,
                    "failed to build embedding for memory turn store; falling back to hash encoder"
                );
                self.publish_memory_stream_event(vec![
                    (
                        "kind".to_string(),
                        "turn_store_embedding_fallback_hash".to_string(),
                    ),
                    ("session_id".to_string(), session_id.to_string()),
                    ("reason".to_string(), error_kind.as_str().to_string()),
                    ("tool_count".to_string(), tool_count.to_string()),
                ])
                .await;
                repaired_fallback
            }
        }
    }

    async fn handle_turn_store_failure(&self, session_id: &str, error: &anyhow::Error) {
        tracing::warn!(
            event = SessionEvent::MemoryTurnStoreFailed.as_str(),
            session_id,
            error = %error,
            "failed to store memory episode for turn"
        );
        self.publish_memory_stream_event(vec![
            ("kind".to_string(), "turn_store_failed".to_string()),
            ("session_id".to_string(), session_id.to_string()),
            ("error".to_string(), error.to_string()),
        ])
        .await;
    }
}

pub(super) fn turn_store_outcome(assistant_msg: &str) -> TurnStoreOutcome {
    let label = classify_assistant_outcome(assistant_msg)
        .as_memory_label()
        .to_string();
    let reward = if label == "error" { 0.0 } else { 1.0 };
    TurnStoreOutcome { label, reward }
}

fn find_existing_turn_episode_id(
    store: &EpisodeStore,
    session_id: &str,
    user_msg: &str,
) -> Option<String> {
    let scope_key = Episode::normalize_scope(session_id);
    let normalized_intent = user_msg.trim();
    store
        .get_all()
        .into_iter()
        .rev()
        .find(|episode| {
            episode.scope_key() == scope_key.as_str() && episode.intent.trim() == normalized_intent
        })
        .map(|episode| episode.id)
}

fn build_turn_episode_id(session_id: &str) -> String {
    format!(
        "turn-{}-{}",
        session_id,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    )
}
