//! Runtime configuration constants shared by Wendao resolver modules.

/// Environment variable that overrides the Valkey URL for link-graph cache state.
pub const LINK_GRAPH_CACHE_VALKEY_URL_ENV: &str = "VALKEY_URL";
/// Environment variable that overrides the Valkey key prefix for link-graph cache state.
pub const LINK_GRAPH_VALKEY_KEY_PREFIX_ENV: &str = "XIUXIAN_WENDAO_LINK_GRAPH_VALKEY_KEY_PREFIX";
/// Environment variable that overrides the Valkey TTL seconds for link-graph cache state.
pub const LINK_GRAPH_VALKEY_TTL_SECONDS_ENV: &str = "XIUXIAN_WENDAO_LINK_GRAPH_VALKEY_TTL_SECONDS";
/// Default Valkey key prefix for link-graph cache-backed runtime data.
pub const DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX: &str = "xiuxian_wendao:link_graph:index";

/// Environment variable that overrides the related-candidate upper bound.
pub const LINK_GRAPH_RELATED_MAX_CANDIDATES_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_RELATED_MAX_CANDIDATES";
/// Environment variable that overrides the related-query partition count.
pub const LINK_GRAPH_RELATED_MAX_PARTITIONS_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_RELATED_MAX_PARTITIONS";
/// Environment variable that overrides the related-query time budget.
pub const LINK_GRAPH_RELATED_TIME_BUDGET_MS_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_RELATED_TIME_BUDGET_MS";
/// Default upper bound for related-candidate collection.
pub const DEFAULT_LINK_GRAPH_RELATED_MAX_CANDIDATES: usize = 4096;
/// Default partition count used for related-query fanout.
pub const DEFAULT_LINK_GRAPH_RELATED_MAX_PARTITIONS: usize = 8;
/// Default budget for related-query execution, in milliseconds.
pub const DEFAULT_LINK_GRAPH_RELATED_TIME_BUDGET_MS: f64 = 250.0;

/// Environment variable that toggles coactivation runtime behavior.
pub const LINK_GRAPH_COACTIVATION_ENABLED_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_COACTIVATION_ENABLED";
/// Environment variable that overrides the coactivation alpha scale.
pub const LINK_GRAPH_COACTIVATION_ALPHA_SCALE_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_COACTIVATION_ALPHA_SCALE";
/// Environment variable that overrides the coactivation neighbor fanout.
pub const LINK_GRAPH_COACTIVATION_MAX_NEIGHBORS_PER_DIRECTION_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_COACTIVATION_MAX_NEIGHBORS_PER_DIRECTION";
/// Environment variable that overrides the coactivation hop count.
pub const LINK_GRAPH_COACTIVATION_MAX_HOPS_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_COACTIVATION_MAX_HOPS";
/// Environment variable that overrides the coactivation propagation cap.
pub const LINK_GRAPH_COACTIVATION_MAX_TOTAL_PROPAGATIONS_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_COACTIVATION_MAX_TOTAL_PROPAGATIONS";
/// Environment variable that overrides the coactivation hop-decay scale.
pub const LINK_GRAPH_COACTIVATION_HOP_DECAY_SCALE_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_COACTIVATION_HOP_DECAY_SCALE";
/// Environment variable that overrides the coactivation touch queue depth.
pub const LINK_GRAPH_COACTIVATION_TOUCH_QUEUE_DEPTH_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_COACTIVATION_TOUCH_QUEUE_DEPTH";
/// Default toggle for coactivation runtime behavior.
pub const DEFAULT_LINK_GRAPH_COACTIVATION_ENABLED: bool = false;
/// Default alpha scale used for coactivation propagation.
pub const DEFAULT_LINK_GRAPH_COACTIVATION_ALPHA_SCALE: f64 = 0.5;
/// Default neighbor fanout per graph direction for coactivation.
pub const DEFAULT_LINK_GRAPH_COACTIVATION_MAX_NEIGHBORS_PER_DIRECTION: usize = 32;
/// Default hop count for coactivation traversal.
pub const DEFAULT_LINK_GRAPH_COACTIVATION_MAX_HOPS: usize = 1;
/// Default hop-decay scale for coactivation propagation.
pub const DEFAULT_LINK_GRAPH_COACTIVATION_HOP_DECAY_SCALE: f64 = 0.5;
/// Default queue depth used when staging coactivation touches.
pub const DEFAULT_LINK_GRAPH_COACTIVATION_TOUCH_QUEUE_DEPTH: usize = 256;
pub(crate) const LINK_GRAPH_AGENTIC_SUGGESTED_LINK_MAX_ENTRIES_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_AGENTIC_SUGGESTED_LINK_MAX_ENTRIES";
pub(crate) const LINK_GRAPH_AGENTIC_SUGGESTED_LINK_TTL_SECONDS_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_AGENTIC_SUGGESTED_LINK_TTL_SECONDS";
pub(crate) const LINK_GRAPH_AGENTIC_SEARCH_INCLUDE_PROVISIONAL_DEFAULT_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_AGENTIC_SEARCH_INCLUDE_PROVISIONAL_DEFAULT";
pub(crate) const LINK_GRAPH_AGENTIC_SEARCH_PROVISIONAL_LIMIT_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_AGENTIC_SEARCH_PROVISIONAL_LIMIT";
pub(crate) const LINK_GRAPH_AGENTIC_EXPANSION_MAX_WORKERS_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_AGENTIC_EXPANSION_MAX_WORKERS";
pub(crate) const LINK_GRAPH_AGENTIC_EXPANSION_MAX_CANDIDATES_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_AGENTIC_EXPANSION_MAX_CANDIDATES";
pub(crate) const LINK_GRAPH_AGENTIC_EXPANSION_MAX_PAIRS_PER_WORKER_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_AGENTIC_EXPANSION_MAX_PAIRS_PER_WORKER";
pub(crate) const LINK_GRAPH_AGENTIC_EXPANSION_TIME_BUDGET_MS_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_AGENTIC_EXPANSION_TIME_BUDGET_MS";
pub(crate) const LINK_GRAPH_AGENTIC_EXECUTION_WORKER_TIME_BUDGET_MS_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_AGENTIC_EXECUTION_WORKER_TIME_BUDGET_MS";
pub(crate) const LINK_GRAPH_AGENTIC_EXECUTION_PERSIST_SUGGESTIONS_DEFAULT_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_AGENTIC_EXECUTION_PERSIST_SUGGESTIONS_DEFAULT";
pub(crate) const LINK_GRAPH_AGENTIC_EXECUTION_PERSIST_RETRY_ATTEMPTS_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_AGENTIC_EXECUTION_PERSIST_RETRY_ATTEMPTS";
pub(crate) const LINK_GRAPH_AGENTIC_EXECUTION_IDEMPOTENCY_SCAN_LIMIT_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_AGENTIC_EXECUTION_IDEMPOTENCY_SCAN_LIMIT";
pub(crate) const LINK_GRAPH_AGENTIC_EXECUTION_RELATION_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_AGENTIC_EXECUTION_RELATION";
pub(crate) const LINK_GRAPH_AGENTIC_EXECUTION_AGENT_ID_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_AGENTIC_EXECUTION_AGENT_ID";
pub(crate) const LINK_GRAPH_AGENTIC_EXECUTION_EVIDENCE_PREFIX_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_AGENTIC_EXECUTION_EVIDENCE_PREFIX";
pub(crate) const LINK_GRAPH_SEMANTIC_IGNITION_BACKEND_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_SEMANTIC_IGNITION_BACKEND";
pub(crate) const LINK_GRAPH_SEMANTIC_IGNITION_VECTOR_STORE_PATH_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_SEMANTIC_IGNITION_VECTOR_STORE_PATH";
pub(crate) const LINK_GRAPH_SEMANTIC_IGNITION_TABLE_NAME_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_SEMANTIC_IGNITION_TABLE_NAME";
pub(crate) const LINK_GRAPH_SEMANTIC_IGNITION_EMBEDDING_BASE_URL_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_SEMANTIC_IGNITION_EMBEDDING_BASE_URL";
pub(crate) const LINK_GRAPH_SEMANTIC_IGNITION_EMBEDDING_MODEL_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_SEMANTIC_IGNITION_EMBEDDING_MODEL";
/// Environment override for retrieval candidate multiplier.
pub const LINK_GRAPH_CANDIDATE_MULTIPLIER_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_CANDIDATE_MULTIPLIER";
/// Environment override for retrieval max source hints.
pub const LINK_GRAPH_MAX_SOURCES_ENV: &str = "XIUXIAN_WENDAO_LINK_GRAPH_MAX_SOURCES";
/// Environment override for hybrid graph sufficiency hit threshold.
pub const LINK_GRAPH_HYBRID_MIN_HITS_ENV: &str = "XIUXIAN_WENDAO_LINK_GRAPH_HYBRID_MIN_HITS";
/// Environment override for hybrid graph sufficiency score threshold.
pub const LINK_GRAPH_HYBRID_MIN_TOP_SCORE_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_HYBRID_MIN_TOP_SCORE";
/// Environment override for graph rows per source hint.
pub const LINK_GRAPH_ROWS_PER_SOURCE_ENV: &str = "XIUXIAN_WENDAO_LINK_GRAPH_ROWS_PER_SOURCE";

pub(crate) const DEFAULT_LINK_GRAPH_AGENTIC_SUGGESTED_LINK_MAX_ENTRIES: usize = 2000;
pub(crate) const DEFAULT_LINK_GRAPH_AGENTIC_SEARCH_PROVISIONAL_LIMIT: usize = 50;
pub(crate) const DEFAULT_LINK_GRAPH_AGENTIC_EXPANSION_MAX_WORKERS: usize = 4;
pub(crate) const DEFAULT_LINK_GRAPH_AGENTIC_EXPANSION_MAX_CANDIDATES: usize = 256;
pub(crate) const DEFAULT_LINK_GRAPH_AGENTIC_EXPANSION_MAX_PAIRS_PER_WORKER: usize = 128;
pub(crate) const DEFAULT_LINK_GRAPH_AGENTIC_EXPANSION_TIME_BUDGET_MS: f64 = 250.0;
pub(crate) const DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_WORKER_TIME_BUDGET_MS: f64 = 120.0;
pub(crate) const DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_PERSIST_SUGGESTIONS_DEFAULT: bool = false;
pub(crate) const DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_PERSIST_RETRY_ATTEMPTS: usize = 2;
pub(crate) const DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_IDEMPOTENCY_SCAN_LIMIT: usize = 2000;
pub(crate) const DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_RELATION: &str = "related_to";
pub(crate) const DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_AGENT_ID: &str = "qianhuan-architect";
pub(crate) const DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_EVIDENCE_PREFIX: &str =
    "agentic expansion bridge candidate";
pub(crate) const DEFAULT_LINK_GRAPH_SEMANTIC_IGNITION_BACKEND: &str = "disabled";
/// Default retrieval candidate multiplier.
pub const DEFAULT_LINK_GRAPH_CANDIDATE_MULTIPLIER: usize = 4;
/// Default maximum number of source hints.
pub const DEFAULT_LINK_GRAPH_MAX_SOURCES: usize = 8;
/// Default graph sufficiency hit threshold.
pub const DEFAULT_LINK_GRAPH_HYBRID_MIN_HITS: usize = 2;
/// Default graph sufficiency score threshold.
pub const DEFAULT_LINK_GRAPH_HYBRID_MIN_TOP_SCORE: f64 = 0.25;
/// Default graph rows requested per source hint.
pub const DEFAULT_LINK_GRAPH_ROWS_PER_SOURCE: usize = 8;
