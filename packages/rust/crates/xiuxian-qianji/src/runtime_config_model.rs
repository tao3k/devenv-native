//! Runtime config model surface for `xiuxian-qianji`.

use super::constants::{
    DEFAULT_MEMORY_PROMOTION_GRAPH_DIMENSION, DEFAULT_MEMORY_PROMOTION_GRAPH_SCOPE,
    DEFAULT_MEMORY_PROMOTION_PERSIST, DEFAULT_MEMORY_PROMOTION_PERSIST_BEST_EFFORT,
    DEFAULT_SERVER_BIND_ADDR, DEFAULT_SERVER_REQUIRE_VALKEY_READY,
    DEFAULT_WORKFLOW_STATE_DUCKDB_RELATIVE_PATH,
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

/// Resolved runtime config for local no-server workflow-state persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiRuntimeWorkflowStateConfig {
    /// Effective local `DuckDB` database path for no-server workflow-state snapshots.
    pub local_duckdb_path: PathBuf,
}

impl Default for QianjiRuntimeWorkflowStateConfig {
    fn default() -> Self {
        Self {
            local_duckdb_path: PathBuf::from(DEFAULT_WORKFLOW_STATE_DUCKDB_RELATIVE_PATH),
        }
    }
}

/// Resolved runtime config for the `qianji-server` daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiRuntimeServerConfig {
    /// Effective socket bind address.
    pub bind_addr: String,
    /// Whether `qianji-server` must ping Valkey before binding.
    pub require_valkey_ready: bool,
}

impl Default for QianjiRuntimeServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: DEFAULT_SERVER_BIND_ADDR.to_string(),
            require_valkey_ready: DEFAULT_SERVER_REQUIRE_VALKEY_READY,
        }
    }
}

/// Explicit runtime environment used by the resolver (test-friendly).
#[derive(Debug, Default, Clone)]
/// Semantic field boundary: this public DTO preserves externally serialized field names.
pub struct QianjiRuntimeEnv {
    /// Optional project root override.
    pub prj_root: Option<PathBuf>,
    /// Optional config-home override.
    pub prj_config_home: Option<PathBuf>,
    /// Optional data-home override.
    pub prj_data_home: Option<PathBuf>,
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
    /// Optional `QIANJI_WORKFLOW_STATE_DUCKDB_PATH` override for local no-server state.
    pub qianji_workflow_state_duckdb_path: Option<PathBuf>,
    /// Optional `QIANJI_SERVER_BIND_ADDR` override for the HTTP service.
    pub qianji_server_bind_addr: Option<String>,
    /// Optional `QIANJI_SERVER_REQUIRE_VALKEY_READY` override.
    pub qianji_server_require_valkey_ready: Option<bool>,
    /// Optional values for arbitrary env keys (used for `api_key_env` lookups).
    pub extra_env: Vec<(String, String)>,
}
