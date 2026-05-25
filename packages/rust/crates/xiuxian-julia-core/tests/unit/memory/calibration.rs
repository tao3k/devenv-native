use super::{
    MEMORY_JULIA_CALIBRATION_REQUEST_COLUMNS, MemoryJuliaCalibrationArtifactRow,
    MemoryJuliaCalibrationRequestRow, build_memory_julia_calibration_request_batch,
    decode_memory_julia_calibration_artifact_rows, memory_julia_calibration_response_schema,
    validate_memory_julia_calibration_response_batch,
};
use arrow::array::StringArray;
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

fn sample_request_row() -> MemoryJuliaCalibrationRequestRow {
    MemoryJuliaCalibrationRequestRow {
        calibration_job_id: "job-1".to_string(),
        scenario_pack: Some("searchinfra".to_string()),
        dataset_ref: "s3://bucket/memory-calibration.parquet".to_string(),
        objective: "maximize_recall_precision_balance".to_string(),
        hyperparam_config: "{\"grid\":{\"lambda\":[0.4,0.6,0.8]}}".to_string(),
    }
}

fn sample_artifact_row() -> MemoryJuliaCalibrationArtifactRow {
    MemoryJuliaCalibrationArtifactRow {
        calibration_job_id: "job-1".to_string(),
        scenario_pack: Some("searchinfra".to_string()),
        artifact_ref: "memory://calibration/job-1".to_string(),
        summary_metrics: "{\"auc\":0.92}".to_string(),
        recommended_thresholds: "{\"promote_to_working_knowledge\":0.79}".to_string(),
        recommended_weights: "{\"lambda\":0.64}".to_string(),
        schema_version: "v1".to_string(),
    }
}

#[test]
fn build_memory_julia_calibration_request_batch_accepts_valid_rows() {
    let batch = build_memory_julia_calibration_request_batch(&[sample_request_row()])
        .unwrap_or_else(|error| panic!("request batch should build: {error}"));
    assert_eq!(batch.num_rows(), 1);
    assert_eq!(
        batch
            .schema()
            .fields()
            .iter()
            .map(|field| field.name().clone())
            .collect::<Vec<_>>(),
        MEMORY_JULIA_CALIBRATION_REQUEST_COLUMNS
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
    );
    assert!(batch.column_by_name("hyperparam_config").is_some());
}

#[test]
fn build_memory_julia_calibration_request_batch_rejects_blank_payloads() {
    let mut row = sample_request_row();
    row.hyperparam_config = "   ".to_string();
    let Err(error) = build_memory_julia_calibration_request_batch(&[row]) else {
        panic!("blank hyperparameter config should fail");
    };
    assert!(error.to_string().contains("blank value"));
}

#[test]
fn decode_memory_julia_calibration_artifact_rows_decodes_valid_batches() {
    let expected = sample_artifact_row();
    let batch = RecordBatch::try_new(
        memory_julia_calibration_response_schema(),
        vec![
            Arc::new(StringArray::from(vec![
                expected.calibration_job_id.as_str(),
            ])),
            Arc::new(StringArray::from(vec![expected.scenario_pack.as_deref()])),
            Arc::new(StringArray::from(vec![expected.artifact_ref.as_str()])),
            Arc::new(StringArray::from(vec![expected.summary_metrics.as_str()])),
            Arc::new(StringArray::from(vec![
                expected.recommended_thresholds.as_str(),
            ])),
            Arc::new(StringArray::from(vec![
                expected.recommended_weights.as_str(),
            ])),
            Arc::new(StringArray::from(vec![expected.schema_version.as_str()])),
        ],
    )
    .unwrap_or_else(|error| panic!("response batch should build: {error}"));

    let rows = decode_memory_julia_calibration_artifact_rows(&[batch])
        .unwrap_or_else(|error| panic!("artifact rows should decode: {error}"));
    assert_eq!(rows, vec![expected]);
}

#[test]
fn validate_memory_julia_calibration_response_batch_rejects_blank_artifact_ref() {
    let batch = RecordBatch::try_new(
        memory_julia_calibration_response_schema(),
        vec![
            Arc::new(StringArray::from(vec!["job-1"])),
            Arc::new(StringArray::from(vec![Some("searchinfra")])),
            Arc::new(StringArray::from(vec!["  "])),
            Arc::new(StringArray::from(vec!["{\"auc\":0.91}"])),
            Arc::new(StringArray::from(vec![
                "{\"promote_to_working_knowledge\":0.8}",
            ])),
            Arc::new(StringArray::from(vec!["{\"lambda\":0.61}"])),
            Arc::new(StringArray::from(vec!["v1"])),
        ],
    )
    .unwrap_or_else(|error| panic!("response batch should build: {error}"));

    let Err(error) = validate_memory_julia_calibration_response_batch(&batch) else {
        panic!("blank artifact ref should fail");
    };
    assert!(error.contains("blank value"));
}
