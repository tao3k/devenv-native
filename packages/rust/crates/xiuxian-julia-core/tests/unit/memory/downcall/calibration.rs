use super::fetch_calibration_artifact_rows_from_inputs;
use crate::memory::host::MemoryCalibrationInputs;
use crate::memory::test_support::{
    calibration_response_batch, runtime_for_test, spawn_memory_service,
};

fn sample_inputs() -> Vec<MemoryCalibrationInputs> {
    vec![MemoryCalibrationInputs {
        calibration_job_id: "calibration-1".to_string(),
        scenario_pack: Some("searchinfra".to_string()),
        dataset_ref: "dataset://memory/searchinfra/latest".to_string(),
        objective: "maximize_precision".to_string(),
        hyperparam_config: "{\"max_iter\":32}".to_string(),
    }]
}

#[tokio::test]
async fn fetch_calibration_artifact_rows_from_inputs_roundtrips() {
    let route = "/memory/calibration";
    let (base_url, server) = spawn_memory_service(calibration_response_batch()).await;
    let runtime = runtime_for_test(base_url, route);

    let rows = fetch_calibration_artifact_rows_from_inputs(&runtime, &sample_inputs())
        .await
        .unwrap_or_else(|error| panic!("calibration downcall should succeed: {error}"));

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].artifact_ref, "artifact://memory/calibration-1");

    server.abort();
}
