//! Julia episodic-recall transport columns and row types.

mod contract;

pub use contract::{
    build_memory_julia_episodic_recall_request_batch,
    decode_memory_julia_episodic_recall_score_rows, memory_julia_episodic_recall_request_schema,
    memory_julia_episodic_recall_response_schema,
    validate_memory_julia_episodic_recall_request_batch,
    validate_memory_julia_episodic_recall_request_batches,
    validate_memory_julia_episodic_recall_request_schema,
    validate_memory_julia_episodic_recall_response_batch,
    validate_memory_julia_episodic_recall_response_batches,
    validate_memory_julia_episodic_recall_response_schema,
};

/// Request column carrying the host query id.
pub const MEMORY_JULIA_EPISODIC_RECALL_QUERY_ID_COLUMN: &str = "query_id";
/// Request column carrying the scenario pack.
pub const MEMORY_JULIA_EPISODIC_RECALL_SCENARIO_PACK_COLUMN: &str = "scenario_pack";
/// Request column carrying the logical memory scope.
pub const MEMORY_JULIA_EPISODIC_RECALL_SCOPE_COLUMN: &str = "scope";
/// Request column carrying the raw query text when available.
pub const MEMORY_JULIA_EPISODIC_RECALL_QUERY_TEXT_COLUMN: &str = "query_text";
/// Request column carrying the query embedding.
pub const MEMORY_JULIA_EPISODIC_RECALL_QUERY_EMBEDDING_COLUMN: &str = "query_embedding";
/// Request column carrying the candidate episode id.
pub const MEMORY_JULIA_EPISODIC_RECALL_CANDIDATE_ID_COLUMN: &str = "candidate_id";
/// Request column carrying the candidate intent embedding.
pub const MEMORY_JULIA_EPISODIC_RECALL_INTENT_EMBEDDING_COLUMN: &str = "intent_embedding";
/// Request column carrying the host utility estimate.
pub const MEMORY_JULIA_EPISODIC_RECALL_Q_VALUE_COLUMN: &str = "q_value";
/// Request column carrying the success counter.
pub const MEMORY_JULIA_EPISODIC_RECALL_SUCCESS_COUNT_COLUMN: &str = "success_count";
/// Request column carrying the failure counter.
pub const MEMORY_JULIA_EPISODIC_RECALL_FAILURE_COUNT_COLUMN: &str = "failure_count";
/// Request column carrying the retrieval counter.
pub const MEMORY_JULIA_EPISODIC_RECALL_RETRIEVAL_COUNT_COLUMN: &str = "retrieval_count";
/// Request column carrying the creation timestamp.
pub const MEMORY_JULIA_EPISODIC_RECALL_CREATED_AT_MS_COLUMN: &str = "created_at_ms";
/// Request column carrying the last-update timestamp.
pub const MEMORY_JULIA_EPISODIC_RECALL_UPDATED_AT_MS_COLUMN: &str = "updated_at_ms";
/// Request column carrying the semantic-fusion tuning knob.
pub const MEMORY_JULIA_EPISODIC_RECALL_K1_COLUMN: &str = "k1";
/// Request column carrying the utility-fusion tuning knob.
pub const MEMORY_JULIA_EPISODIC_RECALL_K2_COLUMN: &str = "k2";
/// Request column carrying the fusion lambda.
pub const MEMORY_JULIA_EPISODIC_RECALL_LAMBDA_COLUMN: &str = "lambda";
/// Request column carrying the minimum score cutoff.
pub const MEMORY_JULIA_EPISODIC_RECALL_MIN_SCORE_COLUMN: &str = "min_score";

/// Response column carrying the semantic score.
pub const MEMORY_JULIA_EPISODIC_RECALL_SEMANTIC_SCORE_COLUMN: &str = "semantic_score";
/// Response column carrying the utility score.
pub const MEMORY_JULIA_EPISODIC_RECALL_UTILITY_SCORE_COLUMN: &str = "utility_score";
/// Response column carrying the fused final score.
pub const MEMORY_JULIA_EPISODIC_RECALL_FINAL_SCORE_COLUMN: &str = "final_score";
/// Response column carrying the confidence score.
pub const MEMORY_JULIA_EPISODIC_RECALL_CONFIDENCE_COLUMN: &str = "confidence";
/// Response column carrying the ranking reason.
pub const MEMORY_JULIA_EPISODIC_RECALL_RANKING_REASON_COLUMN: &str = "ranking_reason";
/// Response column carrying the retrieval mode.
pub const MEMORY_JULIA_EPISODIC_RECALL_RETRIEVAL_MODE_COLUMN: &str = "retrieval_mode";
/// Response column carrying the physical schema version echoed by the provider.
pub const MEMORY_JULIA_EPISODIC_RECALL_SCHEMA_VERSION_COLUMN: &str = "schema_version";

/// Canonical request column order for the staged episodic recall profile.
pub const MEMORY_JULIA_EPISODIC_RECALL_REQUEST_COLUMNS: [&str; 17] = [
    MEMORY_JULIA_EPISODIC_RECALL_QUERY_ID_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_SCENARIO_PACK_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_SCOPE_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_QUERY_TEXT_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_QUERY_EMBEDDING_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_CANDIDATE_ID_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_INTENT_EMBEDDING_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_Q_VALUE_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_SUCCESS_COUNT_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_FAILURE_COUNT_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_RETRIEVAL_COUNT_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_CREATED_AT_MS_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_UPDATED_AT_MS_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_K1_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_K2_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_LAMBDA_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_MIN_SCORE_COLUMN,
];

/// Canonical response column order for the staged episodic recall profile.
pub const MEMORY_JULIA_EPISODIC_RECALL_RESPONSE_COLUMNS: [&str; 9] = [
    MEMORY_JULIA_EPISODIC_RECALL_QUERY_ID_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_CANDIDATE_ID_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_SEMANTIC_SCORE_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_UTILITY_SCORE_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_FINAL_SCORE_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_CONFIDENCE_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_RANKING_REASON_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_RETRIEVAL_MODE_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_SCHEMA_VERSION_COLUMN,
];

/// One typed request row for the staged episodic recall profile.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryJuliaEpisodicRecallRequestRow {
    /// Host query id used as the join key across candidates.
    pub query_id: String,
    /// Optional scenario pack forwarded into the Julia compute lane.
    pub scenario_pack: Option<String>,
    /// Logical scope of the memory candidate.
    pub scope: String,
    /// Optional raw query text.
    pub query_text: Option<String>,
    /// Semantic embedding of the query.
    pub query_embedding: Vec<f32>,
    /// Stable candidate episode id.
    pub candidate_id: String,
    /// Semantic embedding of the candidate intent.
    pub intent_embedding: Vec<f32>,
    /// Host utility estimate.
    pub q_value: f32,
    /// Number of successful recalls.
    pub success_count: u32,
    /// Number of failed recalls.
    pub failure_count: u32,
    /// Number of total retrievals.
    pub retrieval_count: u32,
    /// Host creation timestamp.
    pub created_at_ms: i64,
    /// Host update timestamp.
    pub updated_at_ms: i64,
    /// Semantic recall tuning weight.
    pub k1: f32,
    /// Utility rerank tuning weight.
    pub k2: f32,
    /// Fusion lambda.
    pub lambda: f32,
    /// Minimum accepted score.
    pub min_score: f32,
}

/// One decoded score row from the staged episodic recall profile.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryJuliaEpisodicRecallScoreRow {
    /// Host query id echoed by the provider.
    pub query_id: String,
    /// Candidate episode id echoed by the provider.
    pub candidate_id: String,
    /// Semantic score produced by Julia.
    pub semantic_score: f32,
    /// Utility score produced by Julia.
    pub utility_score: f32,
    /// Final fused score produced by Julia.
    pub final_score: f32,
    /// Confidence score in `[0, 1]`.
    pub confidence: f32,
    /// Optional ranking reason string.
    pub ranking_reason: Option<String>,
    /// Optional retrieval mode string.
    pub retrieval_mode: Option<String>,
    /// Physical schema version echoed by the provider.
    pub schema_version: String,
}
