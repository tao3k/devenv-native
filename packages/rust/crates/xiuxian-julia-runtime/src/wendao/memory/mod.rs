//! Memory downcall contracts between Rust-owned memory facts and Julia compute lanes.

mod calibration;
/// Plugin-owned composition helpers that combine Rust host inputs with
/// memory-family Julia downcalls.
pub mod downcall;
mod episodic_recall;
mod gate_score;
/// Plugin-owned host-adapter helpers over Rust memory-engine read models and evidence.
pub mod host;
mod manifest;
mod metadata;
mod plan_tuning;
mod profile;
mod runtime;
/// Runtime-facing Flight transport helpers for the memory-family Julia lane.
pub mod transport;

pub use calibration::{
    MEMORY_JULIA_CALIBRATION_ARTIFACT_REF_COLUMN, MEMORY_JULIA_CALIBRATION_DATASET_REF_COLUMN,
    MEMORY_JULIA_CALIBRATION_HYPERPARAM_CONFIG_COLUMN, MEMORY_JULIA_CALIBRATION_JOB_ID_COLUMN,
    MEMORY_JULIA_CALIBRATION_OBJECTIVE_COLUMN,
    MEMORY_JULIA_CALIBRATION_RECOMMENDED_THRESHOLDS_COLUMN,
    MEMORY_JULIA_CALIBRATION_RECOMMENDED_WEIGHTS_COLUMN, MEMORY_JULIA_CALIBRATION_REQUEST_COLUMNS,
    MEMORY_JULIA_CALIBRATION_RESPONSE_COLUMNS, MEMORY_JULIA_CALIBRATION_SCENARIO_PACK_COLUMN,
    MEMORY_JULIA_CALIBRATION_SCHEMA_VERSION_COLUMN,
    MEMORY_JULIA_CALIBRATION_SUMMARY_METRICS_COLUMN, MemoryJuliaCalibrationArtifactRow,
    MemoryJuliaCalibrationRequestRow, build_memory_julia_calibration_request_batch,
    decode_memory_julia_calibration_artifact_rows, memory_julia_calibration_request_schema,
    memory_julia_calibration_response_schema, validate_memory_julia_calibration_request_batch,
    validate_memory_julia_calibration_request_batches,
    validate_memory_julia_calibration_request_schema,
    validate_memory_julia_calibration_response_batch,
    validate_memory_julia_calibration_response_batches,
    validate_memory_julia_calibration_response_schema,
};
pub use episodic_recall::{
    MEMORY_JULIA_EPISODIC_RECALL_CANDIDATE_ID_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_CONFIDENCE_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_CREATED_AT_MS_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_FAILURE_COUNT_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_FINAL_SCORE_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_INTENT_EMBEDDING_COLUMN, MEMORY_JULIA_EPISODIC_RECALL_K1_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_K2_COLUMN, MEMORY_JULIA_EPISODIC_RECALL_LAMBDA_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_MIN_SCORE_COLUMN, MEMORY_JULIA_EPISODIC_RECALL_Q_VALUE_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_QUERY_EMBEDDING_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_QUERY_ID_COLUMN, MEMORY_JULIA_EPISODIC_RECALL_QUERY_TEXT_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_RANKING_REASON_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_REQUEST_COLUMNS, MEMORY_JULIA_EPISODIC_RECALL_RESPONSE_COLUMNS,
    MEMORY_JULIA_EPISODIC_RECALL_RETRIEVAL_COUNT_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_RETRIEVAL_MODE_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_SCENARIO_PACK_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_SCHEMA_VERSION_COLUMN, MEMORY_JULIA_EPISODIC_RECALL_SCOPE_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_SEMANTIC_SCORE_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_SUCCESS_COUNT_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_UPDATED_AT_MS_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_UTILITY_SCORE_COLUMN, MemoryJuliaEpisodicRecallRequestRow,
    MemoryJuliaEpisodicRecallScoreRow, build_memory_julia_episodic_recall_request_batch,
    decode_memory_julia_episodic_recall_score_rows, memory_julia_episodic_recall_request_schema,
    memory_julia_episodic_recall_response_schema,
    validate_memory_julia_episodic_recall_request_batch,
    validate_memory_julia_episodic_recall_request_batches,
    validate_memory_julia_episodic_recall_request_schema,
    validate_memory_julia_episodic_recall_response_batch,
    validate_memory_julia_episodic_recall_response_batches,
    validate_memory_julia_episodic_recall_response_schema,
};
pub use gate_score::{
    MEMORY_JULIA_GATE_SCORE_CONFIDENCE_COLUMN, MEMORY_JULIA_GATE_SCORE_CURRENT_STATE_COLUMN,
    MEMORY_JULIA_GATE_SCORE_FAILURE_RATE_COLUMN,
    MEMORY_JULIA_GATE_SCORE_GRAPH_CONSISTENCY_SCORE_COLUMN,
    MEMORY_JULIA_GATE_SCORE_MEMORY_ID_COLUMN, MEMORY_JULIA_GATE_SCORE_NEXT_ACTION_COLUMN,
    MEMORY_JULIA_GATE_SCORE_OMEGA_ALIGNMENT_SCORE_COLUMN, MEMORY_JULIA_GATE_SCORE_Q_VALUE_COLUMN,
    MEMORY_JULIA_GATE_SCORE_REACT_REVALIDATION_SCORE_COLUMN, MEMORY_JULIA_GATE_SCORE_REASON_COLUMN,
    MEMORY_JULIA_GATE_SCORE_REQUEST_COLUMNS, MEMORY_JULIA_GATE_SCORE_RESPONSE_COLUMNS,
    MEMORY_JULIA_GATE_SCORE_SCENARIO_PACK_COLUMN, MEMORY_JULIA_GATE_SCORE_SCHEMA_VERSION_COLUMN,
    MEMORY_JULIA_GATE_SCORE_TTL_SCORE_COLUMN, MEMORY_JULIA_GATE_SCORE_USAGE_COUNT_COLUMN,
    MEMORY_JULIA_GATE_SCORE_UTILITY_SCORE_COLUMN, MEMORY_JULIA_GATE_SCORE_VERDICT_COLUMN,
    MemoryJuliaGateScoreRecommendationRow, MemoryJuliaGateScoreRequestRow,
    build_memory_julia_gate_score_request_batch,
    decode_memory_julia_gate_score_recommendation_rows, memory_julia_gate_score_request_schema,
    memory_julia_gate_score_response_schema, validate_memory_julia_gate_score_request_batch,
    validate_memory_julia_gate_score_request_batches,
    validate_memory_julia_gate_score_request_schema,
    validate_memory_julia_gate_score_response_batch,
    validate_memory_julia_gate_score_response_batches,
    validate_memory_julia_gate_score_response_schema,
};
pub use manifest::{
    MEMORY_JULIA_COMPUTE_MANIFEST_CAPABILITY_ID_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_ENABLED_COLUMN, MEMORY_JULIA_COMPUTE_MANIFEST_FAMILY_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_HEALTH_ROUTE_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_PROFILE_ID_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_REQUEST_SCHEMA_ID_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_RESPONSE_COLUMNS,
    MEMORY_JULIA_COMPUTE_MANIFEST_RESPONSE_SCHEMA_ID_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_ROUTE_COLUMN, MEMORY_JULIA_COMPUTE_MANIFEST_SCENARIO_PACK_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_SCHEMA_VERSION_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_TIMEOUT_SECS_COLUMN, MemoryJuliaComputeManifestRow,
    build_memory_julia_compute_manifest_response_batch, build_memory_julia_compute_manifest_rows,
    decode_memory_julia_compute_manifest_rows, memory_julia_compute_manifest_response_schema,
    validate_memory_julia_compute_manifest_response_batch,
    validate_memory_julia_compute_manifest_response_batches,
    validate_memory_julia_compute_manifest_response_schema,
};
pub use plan_tuning::{
    MEMORY_JULIA_PLAN_TUNING_CONFIDENCE_COLUMN, MEMORY_JULIA_PLAN_TUNING_CURRENT_K1_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_CURRENT_K2_COLUMN, MEMORY_JULIA_PLAN_TUNING_CURRENT_LAMBDA_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_CURRENT_MAX_CONTEXT_CHARS_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_CURRENT_MIN_SCORE_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_FEEDBACK_BIAS_COLUMN, MEMORY_JULIA_PLAN_TUNING_NEXT_K1_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_NEXT_K2_COLUMN, MEMORY_JULIA_PLAN_TUNING_NEXT_LAMBDA_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_NEXT_MAX_CONTEXT_CHARS_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_NEXT_MIN_SCORE_COLUMN, MEMORY_JULIA_PLAN_TUNING_REASON_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_RECENT_FAILURE_RATE_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_RECENT_LATENCY_MS_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_RECENT_SUCCESS_RATE_COLUMN, MEMORY_JULIA_PLAN_TUNING_REQUEST_COLUMNS,
    MEMORY_JULIA_PLAN_TUNING_RESPONSE_COLUMNS, MEMORY_JULIA_PLAN_TUNING_SCENARIO_PACK_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_SCHEMA_VERSION_COLUMN, MEMORY_JULIA_PLAN_TUNING_SCOPE_COLUMN,
    MemoryJuliaPlanTuningAdviceRow, MemoryJuliaPlanTuningRequestRow,
    build_memory_julia_plan_tuning_request_batch, decode_memory_julia_plan_tuning_advice_rows,
    memory_julia_plan_tuning_request_schema, memory_julia_plan_tuning_response_schema,
    validate_memory_julia_plan_tuning_request_batch,
    validate_memory_julia_plan_tuning_request_batches,
    validate_memory_julia_plan_tuning_request_schema,
    validate_memory_julia_plan_tuning_response_batch,
    validate_memory_julia_plan_tuning_response_batches,
    validate_memory_julia_plan_tuning_response_schema,
};
pub use profile::{
    MEMORY_JULIA_COMPUTE_CALIBRATION_PROFILE_ID, MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_PROFILE_ID,
    MEMORY_JULIA_COMPUTE_FAMILY_ID, MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID,
    MEMORY_JULIA_COMPUTE_PLAN_TUNING_PROFILE_ID, MemoryJuliaComputeProfile,
};
pub use runtime::{build_memory_julia_compute_binding, build_memory_julia_compute_bindings};
pub use transport::{
    build_memory_julia_compute_flight_transport_client,
    fetch_memory_julia_calibration_artifact_rows, fetch_memory_julia_episodic_recall_score_rows,
    fetch_memory_julia_gate_score_recommendation_rows, fetch_memory_julia_plan_tuning_advice_rows,
    process_memory_julia_compute_flight_batches,
    process_memory_julia_compute_flight_batches_for_runtime,
    validate_memory_julia_compute_request_batches, validate_memory_julia_compute_response_batches,
};
