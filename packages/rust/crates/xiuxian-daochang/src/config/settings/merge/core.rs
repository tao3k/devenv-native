use crate::config::settings::types::{
    AgentSettings, EmbeddingSettings, InferenceSettings, MemorySettings, MistralSettings,
    RuntimeSettings, SessionSettings, ToolRuntimeSettings,
};

impl RuntimeSettings {
    pub(in super::super) fn merge(self, overlay: Self) -> Self {
        Self {
            agent: self.agent.merge(overlay.agent),
            inference: self.inference.merge(overlay.inference),
            tool_runtime: self.tool_runtime.merge(overlay.tool_runtime),
            telegram: self.telegram.merge(overlay.telegram),
            discord: self.discord.merge(overlay.discord),
            session: self.session.merge(overlay.session),
            embedding: self.embedding.merge(overlay.embedding),
            memory: self.memory.merge(overlay.memory),
            mistral: self.mistral.merge(overlay.mistral),
        }
    }
}

impl AgentSettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            llm_backend: overlay.llm_backend.or(self.llm_backend),
            agenda_validation_policy: overlay
                .agenda_validation_policy
                .or(self.agenda_validation_policy),
        }
    }
}

impl InferenceSettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            provider: overlay.provider.or(self.provider),
            api_key: overlay.api_key.or(self.api_key),
            base_url: overlay.base_url.or(self.base_url),
            wire_api: overlay.wire_api.or(self.wire_api),
            model: overlay.model.or(self.model),
            timeout: overlay.timeout.or(self.timeout),
            max_tokens: overlay.max_tokens.or(self.max_tokens),
            max_in_flight: overlay.max_in_flight.or(self.max_in_flight),
        }
    }
}

impl ToolRuntimeSettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            pool_size: overlay.pool_size.or(self.pool_size),
            handshake_timeout_secs: overlay
                .handshake_timeout_secs
                .or(self.handshake_timeout_secs),
            connect_retries: overlay.connect_retries.or(self.connect_retries),
            strict_startup: overlay.strict_startup.or(self.strict_startup),
            connect_retry_backoff_ms: overlay
                .connect_retry_backoff_ms
                .or(self.connect_retry_backoff_ms),
            tool_timeout_secs: overlay.tool_timeout_secs.or(self.tool_timeout_secs),
            list_tools_cache_ttl_ms: overlay
                .list_tools_cache_ttl_ms
                .or(self.list_tools_cache_ttl_ms),
            discover_cache_enabled: overlay
                .discover_cache_enabled
                .or(self.discover_cache_enabled),
            discover_cache_key_prefix: overlay
                .discover_cache_key_prefix
                .or(self.discover_cache_key_prefix),
            discover_cache_ttl_secs: overlay
                .discover_cache_ttl_secs
                .or(self.discover_cache_ttl_secs),
        }
    }
}

impl SessionSettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            window_max_turns: overlay.window_max_turns.or(self.window_max_turns),
            consolidation_threshold_turns: overlay
                .consolidation_threshold_turns
                .or(self.consolidation_threshold_turns),
            consolidation_take_turns: overlay
                .consolidation_take_turns
                .or(self.consolidation_take_turns),
            consolidation_async: overlay.consolidation_async.or(self.consolidation_async),
            context_budget_tokens: overlay.context_budget_tokens.or(self.context_budget_tokens),
            context_budget_reserve_tokens: overlay
                .context_budget_reserve_tokens
                .or(self.context_budget_reserve_tokens),
            context_budget_strategy: overlay
                .context_budget_strategy
                .or(self.context_budget_strategy),
            summary_max_segments: overlay.summary_max_segments.or(self.summary_max_segments),
            summary_max_chars: overlay.summary_max_chars.or(self.summary_max_chars),
            reset_idle_timeout_mins: overlay
                .reset_idle_timeout_mins
                .or(self.reset_idle_timeout_mins),
            message_content_max_chars: overlay
                .message_content_max_chars
                .or(self.message_content_max_chars),
            valkey_url: overlay.valkey_url.or(self.valkey_url),
            redis_prefix: overlay.redis_prefix.or(self.redis_prefix),
            ttl_secs: overlay.ttl_secs.or(self.ttl_secs),
        }
    }
}

impl MemorySettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            path: overlay.path.or(self.path),
            embedding_backend: overlay.embedding_backend.or(self.embedding_backend),
            embedding_base_url: overlay.embedding_base_url.or(self.embedding_base_url),
            embedding_model: overlay.embedding_model.or(self.embedding_model),
            embedding_timeout_ms: overlay.embedding_timeout_ms.or(self.embedding_timeout_ms),
            embedding_timeout_cooldown_ms: overlay
                .embedding_timeout_cooldown_ms
                .or(self.embedding_timeout_cooldown_ms),
            embedding_dim: overlay.embedding_dim.or(self.embedding_dim),
            persistence_backend: overlay.persistence_backend.or(self.persistence_backend),
            persistence_valkey_url: overlay
                .persistence_valkey_url
                .or(self.persistence_valkey_url),
            persistence_key_prefix: overlay
                .persistence_key_prefix
                .or(self.persistence_key_prefix),
            persistence_strict_startup: overlay
                .persistence_strict_startup
                .or(self.persistence_strict_startup),
            recall_credit_enabled: overlay.recall_credit_enabled.or(self.recall_credit_enabled),
            recall_credit_max_candidates: overlay
                .recall_credit_max_candidates
                .or(self.recall_credit_max_candidates),
            decay_enabled: overlay.decay_enabled.or(self.decay_enabled),
            decay_every_turns: overlay.decay_every_turns.or(self.decay_every_turns),
            decay_factor: overlay.decay_factor.or(self.decay_factor),
            gate_promote_threshold: overlay
                .gate_promote_threshold
                .or(self.gate_promote_threshold),
            gate_obsolete_threshold: overlay
                .gate_obsolete_threshold
                .or(self.gate_obsolete_threshold),
            gate_promote_min_usage: overlay
                .gate_promote_min_usage
                .or(self.gate_promote_min_usage),
            gate_obsolete_min_usage: overlay
                .gate_obsolete_min_usage
                .or(self.gate_obsolete_min_usage),
            gate_promote_failure_rate_ceiling: overlay
                .gate_promote_failure_rate_ceiling
                .or(self.gate_promote_failure_rate_ceiling),
            gate_obsolete_failure_rate_floor: overlay
                .gate_obsolete_failure_rate_floor
                .or(self.gate_obsolete_failure_rate_floor),
            gate_promote_min_ttl_score: overlay
                .gate_promote_min_ttl_score
                .or(self.gate_promote_min_ttl_score),
            gate_obsolete_max_ttl_score: overlay
                .gate_obsolete_max_ttl_score
                .or(self.gate_obsolete_max_ttl_score),
            stream_consumer_enabled: overlay
                .stream_consumer_enabled
                .or(self.stream_consumer_enabled),
            stream_name: overlay.stream_name.or(self.stream_name),
            stream_consumer_group: overlay.stream_consumer_group.or(self.stream_consumer_group),
            stream_consumer_name_prefix: overlay
                .stream_consumer_name_prefix
                .or(self.stream_consumer_name_prefix),
            stream_consumer_batch_size: overlay
                .stream_consumer_batch_size
                .or(self.stream_consumer_batch_size),
            stream_consumer_block_ms: overlay
                .stream_consumer_block_ms
                .or(self.stream_consumer_block_ms),
        }
    }
}

impl EmbeddingSettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            backend: overlay.backend.or(self.backend),
            timeout_secs: overlay.timeout_secs.or(self.timeout_secs),
            max_in_flight: overlay.max_in_flight.or(self.max_in_flight),
            batch_max_size: overlay.batch_max_size.or(self.batch_max_size),
            batch_max_concurrency: overlay.batch_max_concurrency.or(self.batch_max_concurrency),
            model: overlay.model.or(self.model),
            litellm_model: overlay.litellm_model.or(self.litellm_model),
            litellm_api_base: overlay.litellm_api_base.or(self.litellm_api_base),
            dimension: overlay.dimension.or(self.dimension),
            client_url: overlay.client_url.or(self.client_url),
        }
    }
}

impl MistralSettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            enabled: overlay.enabled.or(self.enabled),
            auto_start: overlay.auto_start.or(self.auto_start),
            command: overlay.command.or(self.command),
            args: overlay.args.or(self.args),
            base_url: overlay.base_url.or(self.base_url),
            startup_timeout_secs: overlay.startup_timeout_secs.or(self.startup_timeout_secs),
            probe_timeout_ms: overlay.probe_timeout_ms.or(self.probe_timeout_ms),
            probe_interval_ms: overlay.probe_interval_ms.or(self.probe_interval_ms),
            sdk_hf_cache_path: overlay.sdk_hf_cache_path.or(self.sdk_hf_cache_path),
            sdk_hf_revision: overlay.sdk_hf_revision.or(self.sdk_hf_revision),
            sdk_embedding_max_num_seqs: overlay
                .sdk_embedding_max_num_seqs
                .or(self.sdk_embedding_max_num_seqs),
        }
    }
}
