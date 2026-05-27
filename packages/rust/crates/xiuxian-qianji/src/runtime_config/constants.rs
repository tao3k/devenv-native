pub(super) const DEFAULT_MODEL: &str = "deepseek-chat";
pub(super) const DEFAULT_BASE_URL: &str = "https://api.deepseek.com/v1";
pub(super) const DEFAULT_API_KEY_ENV: &str = "DEEPSEEK_API_KEY";
pub(super) const DEFAULT_MEMORY_PROMOTION_GRAPH_SCOPE: &str = "qianji:memory_promotion";
pub(super) const DEFAULT_MEMORY_PROMOTION_GRAPH_DIMENSION: usize = 1024;
pub(super) const DEFAULT_MEMORY_PROMOTION_PERSIST: bool = true;
pub(super) const DEFAULT_MEMORY_PROMOTION_PERSIST_BEST_EFFORT: bool = true;
pub(super) const DEFAULT_SERVER_BIND_ADDR: &str = "127.0.0.1:38130";
pub(super) const DEFAULT_SERVER_REQUIRE_VALKEY_READY: bool = false;
pub(super) const DEFAULT_QIANJI_DATA_NAMESPACE: &str = "xiuxian-qianji";
pub(super) const DEFAULT_WORKFLOW_STATE_DUCKDB_RELATIVE_PATH: &str =
    ".data/xiuxian-qianji/duckdb/workflow-state.duckdb";
pub(super) const WORKFLOW_STATE_DUCKDB_FILE_NAME: &str = "workflow-state.duckdb";
