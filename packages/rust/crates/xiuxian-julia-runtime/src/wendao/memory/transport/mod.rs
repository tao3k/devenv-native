//! Transport owner for Julia memory compute Flight routes.

mod client;
mod fetch;
mod process;

pub use client::build_memory_julia_compute_flight_transport_client;
pub use fetch::{
    fetch_memory_julia_calibration_artifact_rows, fetch_memory_julia_episodic_recall_score_rows,
    fetch_memory_julia_gate_score_recommendation_rows, fetch_memory_julia_plan_tuning_advice_rows,
};
pub use process::{
    process_memory_julia_compute_flight_batches,
    process_memory_julia_compute_flight_batches_for_runtime,
    validate_memory_julia_compute_request_batches, validate_memory_julia_compute_response_batches,
};
