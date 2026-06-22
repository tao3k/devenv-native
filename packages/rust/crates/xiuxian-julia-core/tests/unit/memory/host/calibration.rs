use crate::memory::host::calibration::{
    MemoryCalibrationInputs, build_memory_calibration_request_batch_from_inputs,
    build_memory_calibration_request_rows_from_inputs,
};

fn sample_inputs() -> MemoryCalibrationInputs {
    MemoryCalibrationInputs {
        calibration_job_id: "calibration-searchinfra-001".to_string(),
        scenario_pack: Some("searchinfra".to_string()),
        dataset_ref: "dataset://memory/searchinfra/latest".to_string(),
        objective: "optimize_recall_precision".to_string(),
        hyperparam_config: "{\"max_iter\":32,\"temperature\":0.15}".to_string(),
    }
}

#[test]
fn build_memory_calibration_request_rows_from_inputs_maps_host_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let rows = build_memory_calibration_request_rows_from_inputs(&[sample_inputs()])?;

    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.calibration_job_id, "calibration-searchinfra-001");
    assert_eq!(row.scenario_pack.as_deref(), Some("searchinfra"));
    assert_eq!(row.dataset_ref, "dataset://memory/searchinfra/latest");
    assert_eq!(row.objective, "optimize_recall_precision");
    assert_eq!(
        row.hyperparam_config,
        "{\"max_iter\":32,\"temperature\":0.15}"
    );

    Ok(())
}

#[test]
fn build_memory_calibration_request_batch_from_inputs_materializes_staged_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let mut second = sample_inputs();
    second.calibration_job_id = "calibration-searchinfra-002".to_string();
    second.scenario_pack = None;
    second.dataset_ref = "dataset://memory/searchinfra/canary".to_string();

    let batch = build_memory_calibration_request_batch_from_inputs(&[sample_inputs(), second])?;

    assert_eq!(batch.num_rows(), 2);
    assert_eq!(batch.schema().fields().len(), 5);
    assert!(batch.column_by_name("calibration_job_id").is_some());
    assert!(batch.column_by_name("hyperparam_config").is_some());

    Ok(())
}

#[test]
fn build_memory_calibration_request_batch_from_inputs_rejects_invalid_shape() {
    let mut inputs = sample_inputs();
    inputs.hyperparam_config = "   ".to_string();

    let Err(error) = build_memory_calibration_request_batch_from_inputs(&[inputs]) else {
        panic!("blank hyperparam_config must fail");
    };

    assert!(error.to_string().contains("hyperparam_config"));
}

#[test]
fn build_memory_calibration_request_rows_from_inputs_trims_optional_scenario_pack()
-> Result<(), Box<dyn std::error::Error>> {
    let mut inputs = sample_inputs();
    inputs.scenario_pack = Some("   ".to_string());

    let rows = build_memory_calibration_request_rows_from_inputs(&[inputs])?;

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].scenario_pack, None);

    Ok(())
}
