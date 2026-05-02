use super::Agent;
pub(crate) use xiuxian_llm::embedding::EMBEDDING_SOURCE_UNAVAILABLE;
pub(crate) use xiuxian_llm::embedding::MemoryEmbeddingErrorKind;

impl Agent {
    pub(crate) async fn embedding_for_memory(
        &self,
        intent: &str,
        expected_dim: usize,
    ) -> std::result::Result<Vec<f32>, MemoryEmbeddingErrorKind> {
        self.embedding_for_memory_with_source(intent, expected_dim)
            .await
            .map(|(embedding, _)| embedding)
    }

    pub(crate) async fn embedding_for_memory_with_source(
        &self,
        intent: &str,
        expected_dim: usize,
    ) -> std::result::Result<(Vec<f32>, &'static str), MemoryEmbeddingErrorKind> {
        let Some(runtime) = self.embedding_runtime.as_ref() else {
            self.record_memory_embedding_unavailable_metric().await;
            return Err(MemoryEmbeddingErrorKind::Unavailable);
        };
        let Some(client) = self.embedding_client.as_ref() else {
            self.record_memory_embedding_unavailable_metric().await;
            return Err(MemoryEmbeddingErrorKind::Unavailable);
        };
        let model = self
            .config
            .memory
            .as_ref()
            .and_then(|cfg| cfg.embedding_model.as_deref());

        let result = runtime
            .embed_with_source(intent, expected_dim, |payload| {
                let payload_owned = payload.to_string();
                async move { client.embed_with_model(payload_owned.as_str(), model).await }
            })
            .await;

        match result {
            Ok(value) => {
                self.record_memory_embedding_success_metric().await;
                Ok(value)
            }
            Err(MemoryEmbeddingErrorKind::CooldownActive) => {
                self.record_memory_embedding_cooldown_reject_metric().await;
                Err(MemoryEmbeddingErrorKind::CooldownActive)
            }
            Err(MemoryEmbeddingErrorKind::Timeout) => {
                self.record_memory_embedding_timeout_metric().await;
                Err(MemoryEmbeddingErrorKind::Timeout)
            }
            Err(MemoryEmbeddingErrorKind::Unavailable) => {
                self.record_memory_embedding_unavailable_metric().await;
                Err(MemoryEmbeddingErrorKind::Unavailable)
            }
        }
    }
}
