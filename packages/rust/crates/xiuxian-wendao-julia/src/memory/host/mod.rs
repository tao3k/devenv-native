//! Host-owned memory request staging before Julia transport serialization.

mod calibration;
mod episodic_recall;
mod gate_score;
mod plan_tuning;
mod staging;

pub use calibration::{
    MemoryCalibrationInputs, build_memory_calibration_request_batch_from_inputs,
    build_memory_calibration_request_rows_from_inputs,
};
pub use episodic_recall::{
    EpisodicRecallQueryInputs, build_episodic_recall_request_batch_from_projection,
    build_episodic_recall_request_rows_from_projection,
};
pub use gate_score::{
    MemoryGateScoreEvidenceRow, MemoryGateScoreEvidenceSignals, MemoryGateScoreMemoryId,
    MemoryGateScoreStoreEvidenceInput, build_memory_gate_score_evidence_row_from_episode,
    build_memory_gate_score_evidence_row_from_store,
    build_memory_gate_score_request_batch_from_evidence,
    build_memory_gate_score_request_rows_from_evidence,
};
pub use plan_tuning::{
    MemoryPlanTuningInputs, build_memory_plan_tuning_request_batch_from_inputs,
    build_memory_plan_tuning_request_rows_from_inputs,
};
pub use xiuxian_memory_engine::{
    MemoryLifecycleState, MemoryProjectionRow, MemoryUtilityLedger, RecallPlanTuning,
};
