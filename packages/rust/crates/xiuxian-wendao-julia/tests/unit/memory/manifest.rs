use super::{
    MEMORY_JULIA_COMPUTE_MANIFEST_RESPONSE_COLUMNS, MemoryJuliaComputeManifestRow,
    build_memory_julia_compute_manifest_response_batch, build_memory_julia_compute_manifest_rows,
    decode_memory_julia_compute_manifest_rows, memory_julia_compute_manifest_response_schema,
    validate_memory_julia_compute_manifest_response_batch,
};
use arrow::array::{BooleanArray, StringArray, UInt64Array};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;
use xiuxian_wendao_runtime::config::MemoryJuliaComputeRuntimeConfig;

fn sample_runtime() -> MemoryJuliaComputeRuntimeConfig {
    MemoryJuliaComputeRuntimeConfig {
        enabled: true.into(),
        health_route: Some("/healthz".to_string()),
        scenario_pack: Some("searchinfra".to_string()),
        ..MemoryJuliaComputeRuntimeConfig::default()
    }
}

fn sample_manifest_row() -> MemoryJuliaComputeManifestRow {
    build_memory_julia_compute_manifest_rows(&sample_runtime())
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("staged profile rows should exist"))
}

#[test]
fn build_memory_julia_compute_manifest_rows_materializes_all_profiles() {
    let rows = build_memory_julia_compute_manifest_rows(&sample_runtime());
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].family, "memory");
    assert_eq!(rows[0].capability_id, "episodic_recall");
    assert_eq!(
        rows[0].request_schema_id,
        "memory.episodic_recall.request.v1"
    );
    assert_eq!(rows[3].profile_id, "memory_calibration");
    assert_eq!(rows[3].route, "/memory/calibration");
    assert_eq!(rows[0].health_route.as_deref(), Some("/healthz"));
    assert_eq!(rows[1].scenario_pack.as_deref(), Some("searchinfra"));
    assert!(rows.iter().all(|row| row.enabled.value()));
}

#[test]
fn build_memory_julia_compute_manifest_response_batch_and_decode_roundtrip() {
    let rows = build_memory_julia_compute_manifest_rows(&sample_runtime());
    let batch = build_memory_julia_compute_manifest_response_batch(&rows)
        .unwrap_or_else(|error| panic!("manifest batch should build: {error}"));
    assert_eq!(
        batch
            .schema()
            .fields()
            .iter()
            .map(|field| field.name().clone())
            .collect::<Vec<_>>(),
        MEMORY_JULIA_COMPUTE_MANIFEST_RESPONSE_COLUMNS
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
    );

    let decoded = decode_memory_julia_compute_manifest_rows(&[batch])
        .unwrap_or_else(|error| panic!("manifest rows should decode: {error}"));
    assert_eq!(decoded, rows);
}

#[test]
fn validate_memory_julia_compute_manifest_response_batch_rejects_contract_drift() {
    let row = sample_manifest_row();
    let batch = RecordBatch::try_new(
        memory_julia_compute_manifest_response_schema(),
        vec![
            Arc::new(StringArray::from(vec![row.family.as_str()])),
            Arc::new(StringArray::from(vec!["wrong-capability"])),
            Arc::new(StringArray::from(vec![row.profile_id.as_str()])),
            Arc::new(StringArray::from(vec![row.request_schema_id.as_str()])),
            Arc::new(StringArray::from(vec![row.response_schema_id.as_str()])),
            Arc::new(StringArray::from(vec![row.route.as_str()])),
            Arc::new(StringArray::from(vec![row.health_route.as_deref()])),
            Arc::new(StringArray::from(vec![row.schema_version.as_str()])),
            Arc::new(UInt64Array::from(vec![
                row.timeout_secs.map(|seconds| seconds.value()),
            ])),
            Arc::new(StringArray::from(vec![row.scenario_pack.as_deref()])),
            Arc::new(BooleanArray::from(vec![row.enabled.value()])),
        ],
    )
    .unwrap_or_else(|error| panic!("manifest batch should build: {error}"));

    let Err(error) = validate_memory_julia_compute_manifest_response_batch(&batch) else {
        panic!("contract drift should fail");
    };
    assert!(error.contains("must match staged profile contract"));
}

#[test]
fn validate_memory_julia_compute_manifest_response_batch_rejects_invalid_route() {
    let row = sample_manifest_row();
    let batch = RecordBatch::try_new(
        memory_julia_compute_manifest_response_schema(),
        vec![
            Arc::new(StringArray::from(vec![row.family.as_str()])),
            Arc::new(StringArray::from(vec![row.capability_id.as_str()])),
            Arc::new(StringArray::from(vec![row.profile_id.as_str()])),
            Arc::new(StringArray::from(vec![row.request_schema_id.as_str()])),
            Arc::new(StringArray::from(vec![row.response_schema_id.as_str()])),
            Arc::new(StringArray::from(vec!["/"])),
            Arc::new(StringArray::from(vec![row.health_route.as_deref()])),
            Arc::new(StringArray::from(vec![row.schema_version.as_str()])),
            Arc::new(UInt64Array::from(vec![
                row.timeout_secs.map(|seconds| seconds.value()),
            ])),
            Arc::new(StringArray::from(vec![row.scenario_pack.as_deref()])),
            Arc::new(BooleanArray::from(vec![row.enabled.value()])),
        ],
    )
    .unwrap_or_else(|error| panic!("manifest batch should build: {error}"));

    let Err(error) = validate_memory_julia_compute_manifest_response_batch(&batch) else {
        panic!("invalid route should fail");
    };
    assert!(error.contains("normalized Flight route"));
}
