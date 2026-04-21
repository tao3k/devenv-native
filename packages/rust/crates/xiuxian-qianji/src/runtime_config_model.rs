use super::constants::{
    DEFAULT_MEMORY_PROMOTION_GRAPH_DIMENSION, DEFAULT_MEMORY_PROMOTION_GRAPH_SCOPE,
    DEFAULT_MEMORY_PROMOTION_PERSIST, DEFAULT_MEMORY_PROMOTION_PERSIST_BEST_EFFORT,
};
use std::path::PathBuf;

/// Resolved runtime config for Qianji LLM calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiRuntimeLlmConfig {
    /// Effective model name.
    pub model: String,
    /// Effective OpenAI-compatible base URL.
    pub base_url: String,
    /// Effective API key environment variable name.
    pub api_key_env: String,
    /// Effective OpenAI-compatible wire protocol (`chat_completions` or `responses`).
    pub wire_api: String,
    /// Effective API key value (resolved from environment).
    pub api_key: String,
}

/// Resolved runtime config for native `Wendao` memory-promotion ingestion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiRuntimeWendaoIngesterConfig {
    /// Default graph scope for persisted promotion entities.
    pub graph_scope: String,
    /// Optional context key that can override graph scope at runtime.
    pub graph_scope_key: Option<String>,
    /// Graph dimension metadata passed to `KnowledgeGraph::save_to_valkey`.
    pub graph_dimension: usize,
    /// Whether persistence is enabled by default.
    pub persist: bool,
    /// Whether persistence failures should degrade gracefully by default.
    pub persist_best_effort: bool,
}

impl Default for QianjiRuntimeWendaoIngesterConfig {
    fn default() -> Self {
        Self {
            graph_scope: DEFAULT_MEMORY_PROMOTION_GRAPH_SCOPE.to_string(),
            graph_scope_key: None,
            graph_dimension: DEFAULT_MEMORY_PROMOTION_GRAPH_DIMENSION,
            persist: DEFAULT_MEMORY_PROMOTION_PERSIST,
            persist_best_effort: DEFAULT_MEMORY_PROMOTION_PERSIST_BEST_EFFORT,
        }
    }
}

/// Resolved runtime config for checkpoint persistence and resume state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiRuntimeCheckpointConfig {
    /// Effective Valkey URL used for checkpoint load/save/delete.
    pub valkey_url: String,
}

/// Explicit runtime environment used by the resolver (test-friendly).
#[derive(Debug, Default, Clone)]
pub struct QianjiRuntimeEnv {
    /// Optional project root override.
    pub prj_root: Option<PathBuf>,
    /// Optional config-home override.
    pub prj_config_home: Option<PathBuf>,
    /// Optional explicit qianji config path override.
    pub qianji_config_path: Option<PathBuf>,
    /// Optional `QIANJI_LLM_MODEL` override.
    pub qianji_llm_model: Option<String>,
    /// Optional `QIANJI_LLM_PROVIDER` override.
    pub qianji_llm_provider: Option<String>,
    /// Optional `QIANJI_LLM_WIRE_API` override.
    pub qianji_llm_wire_api: Option<String>,
    /// Optional `OPENAI_API_BASE` override.
    pub openai_api_base: Option<String>,
    /// Optional `OPENAI_API_KEY` override.
    pub openai_api_key: Option<String>,
    /// Optional `QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE` override.
    pub qianji_memory_promotion_graph_scope: Option<String>,
    /// Optional `QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE_KEY` override.
    pub qianji_memory_promotion_graph_scope_key: Option<String>,
    /// Optional `QIANJI_MEMORY_PROMOTION_GRAPH_DIMENSION` override.
    pub qianji_memory_promotion_graph_dimension: Option<usize>,
    /// Optional `QIANJI_MEMORY_PROMOTION_PERSIST` override.
    pub qianji_memory_promotion_persist: Option<bool>,
    /// Optional `QIANJI_MEMORY_PROMOTION_PERSIST_BEST_EFFORT` override.
    pub qianji_memory_promotion_persist_best_effort: Option<bool>,
    /// Optional `QIANJI_VALKEY_URL` override for checkpoint persistence.
    pub qianji_checkpoint_valkey_url: Option<String>,
    /// Optional values for arbitrary env keys (used for `api_key_env` lookups).
    pub extra_env: Vec<(String, String)>,
}
