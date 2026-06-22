//! Public downcall helpers for staging memory requests into Julia compute contracts.

mod calibration;
mod episodic_recall;
mod gate_score;
mod plan_tuning;

pub use calibration::fetch_calibration_artifact_rows_from_inputs;
pub use episodic_recall::fetch_episodic_recall_score_rows_from_projection;
pub use gate_score::fetch_gate_score_recommendation_rows_from_evidence;
pub use plan_tuning::fetch_plan_tuning_advice_rows_from_inputs;
