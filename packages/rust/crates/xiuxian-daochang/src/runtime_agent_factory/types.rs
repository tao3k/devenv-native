use crate::{ContextBudgetStrategy, MemoryConfig};
use xiuxian_llm::embedding::EmbeddingBackendKind;

pub(crate) type RuntimeEmbeddingBackendMode = EmbeddingBackendKind;

pub(crate) struct McpRuntimeOptions {
    pub(crate) pool_size: usize,
    pub(crate) handshake_timeout_secs: u64,
    pub(crate) connect_retries: u32,
    pub(crate) strict_startup: bool,
    pub(crate) connect_retry_backoff_ms: u64,
    pub(crate) tool_timeout_secs: u64,
    pub(crate) list_tools_cache_ttl_ms: u64,
}

pub(super) struct SessionRuntimeOptions {
    pub(super) max_tool_rounds: u32,
    pub(super) window_max_turns: Option<usize>,
    pub(super) consolidation_threshold_turns: Option<usize>,
    pub(super) consolidation_take_turns: usize,
    pub(super) consolidation_async: bool,
    pub(super) context_budget_tokens: Option<usize>,
    pub(super) context_budget_reserve_tokens: usize,
    pub(super) context_budget_strategy: ContextBudgetStrategy,
    pub(super) summary_max_segments: usize,
    pub(super) summary_max_chars: usize,
}

pub(crate) struct MemoryRuntimeOptions {
    pub(crate) config: MemoryConfig,
    pub(crate) embedding_backend_mode: RuntimeEmbeddingBackendMode,
}
