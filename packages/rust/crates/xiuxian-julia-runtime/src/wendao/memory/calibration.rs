//! Julia memory calibration transport contract and row processing.

use std::sync::Arc;

use arrow::array::{Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

/// Request column carrying the calibration job id.
pub const MEMORY_JULIA_CALIBRATION_JOB_ID_COLUMN: &str = "calibration_job_id";
/// Request column carrying the optional scenario pack.
pub const MEMORY_JULIA_CALIBRATION_SCENARIO_PACK_COLUMN: &str = "scenario_pack";
/// Request column carrying the dataset reference.
pub const MEMORY_JULIA_CALIBRATION_DATASET_REF_COLUMN: &str = "dataset_ref";
/// Request column carrying the optimization objective.
pub const MEMORY_JULIA_CALIBRATION_OBJECTIVE_COLUMN: &str = "objective";
/// Request column carrying the hyperparameter config payload.
pub const MEMORY_JULIA_CALIBRATION_HYPERPARAM_CONFIG_COLUMN: &str = "hyperparam_config";

/// Response column carrying the generated artifact reference.
pub const MEMORY_JULIA_CALIBRATION_ARTIFACT_REF_COLUMN: &str = "artifact_ref";
/// Response column carrying summary metrics payload.
pub const MEMORY_JULIA_CALIBRATION_SUMMARY_METRICS_COLUMN: &str = "summary_metrics";
/// Response column carrying recommended thresholds payload.
pub const MEMORY_JULIA_CALIBRATION_RECOMMENDED_THRESHOLDS_COLUMN: &str = "recommended_thresholds";
/// Response column carrying recommended weights payload.
pub const MEMORY_JULIA_CALIBRATION_RECOMMENDED_WEIGHTS_COLUMN: &str = "recommended_weights";
/// Response column carrying the physical schema version echoed by the provider.
pub const MEMORY_JULIA_CALIBRATION_SCHEMA_VERSION_COLUMN: &str = "schema_version";

/// Canonical request column order for the staged calibration profile.
pub const MEMORY_JULIA_CALIBRATION_REQUEST_COLUMNS: [&str; 5] = [
    MEMORY_JULIA_CALIBRATION_JOB_ID_COLUMN,
    MEMORY_JULIA_CALIBRATION_SCENARIO_PACK_COLUMN,
    MEMORY_JULIA_CALIBRATION_DATASET_REF_COLUMN,
    MEMORY_JULIA_CALIBRATION_OBJECTIVE_COLUMN,
    MEMORY_JULIA_CALIBRATION_HYPERPARAM_CONFIG_COLUMN,
];

/// Canonical response column order for the staged calibration profile.
pub const MEMORY_JULIA_CALIBRATION_RESPONSE_COLUMNS: [&str; 7] = [
    MEMORY_JULIA_CALIBRATION_JOB_ID_COLUMN,
    MEMORY_JULIA_CALIBRATION_SCENARIO_PACK_COLUMN,
    MEMORY_JULIA_CALIBRATION_ARTIFACT_REF_COLUMN,
    MEMORY_JULIA_CALIBRATION_SUMMARY_METRICS_COLUMN,
    MEMORY_JULIA_CALIBRATION_RECOMMENDED_THRESHOLDS_COLUMN,
    MEMORY_JULIA_CALIBRATION_RECOMMENDED_WEIGHTS_COLUMN,
    MEMORY_JULIA_CALIBRATION_SCHEMA_VERSION_COLUMN,
];

/// One typed request row for the staged calibration profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryJuliaCalibrationRequestRow {
    /// Stable calibration job id.
    pub calibration_job_id: String,
    /// Optional scenario pack forwarded into Julia.
    pub scenario_pack: Option<String>,
    /// Dataset reference used by the calibration job.
    pub dataset_ref: String,
    /// Optimization objective label.
    pub objective: String,
    /// Serialized hyperparameter config payload.
    pub hyperparam_config: String,
}

/// One decoded artifact row from the staged calibration profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryJuliaCalibrationArtifactRow {
    /// Stable calibration job id echoed by the provider.
    pub calibration_job_id: String,
    /// Optional scenario pack echoed by the provider.
    pub scenario_pack: Option<String>,
    /// Generated artifact reference.
    pub artifact_ref: String,
    /// Serialized summary metrics payload.
    pub summary_metrics: String,
    /// Serialized recommended thresholds payload.
    pub recommended_thresholds: String,
    /// Serialized recommended weights payload.
    pub recommended_weights: String,
    /// Physical schema version echoed by the provider.
    pub schema_version: String,
}

/// Build the staged calibration request schema.
#[must_use]
pub fn memory_julia_calibration_request_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new(
            MEMORY_JULIA_CALIBRATION_JOB_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_CALIBRATION_SCENARIO_PACK_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            MEMORY_JULIA_CALIBRATION_DATASET_REF_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_CALIBRATION_OBJECTIVE_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_CALIBRATION_HYPERPARAM_CONFIG_COLUMN,
            DataType::Utf8,
            false,
        ),
    ]))
}

/// Build the staged calibration response schema.
#[must_use]
pub fn memory_julia_calibration_response_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new(
            MEMORY_JULIA_CALIBRATION_JOB_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_CALIBRATION_SCENARIO_PACK_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            MEMORY_JULIA_CALIBRATION_ARTIFACT_REF_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_CALIBRATION_SUMMARY_METRICS_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_CALIBRATION_RECOMMENDED_THRESHOLDS_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_CALIBRATION_RECOMMENDED_WEIGHTS_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_CALIBRATION_SCHEMA_VERSION_COLUMN,
            DataType::Utf8,
            false,
        ),
    ]))
}

/// Build one staged calibration request batch from typed rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the request rows violate the staged
/// calibration contract.
pub fn build_memory_julia_calibration_request_batch(
    rows: &[MemoryJuliaCalibrationRequestRow],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let batch = RecordBatch::try_new(
        memory_julia_calibration_request_schema(),
        vec![
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.calibration_job_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.scenario_pack.as_deref())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.dataset_ref.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.objective.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.hyperparam_config.as_str())
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(|error| calibration_contract_error(&error.to_string()))?;

    validate_memory_julia_calibration_request_batch(&batch)
        .map_err(|error| calibration_contract_error(&error))?;
    Ok(batch)
}

/// Validate one staged calibration request schema.
///
/// # Errors
///
/// Returns an error when the schema does not match the staged calibration
/// contract.
pub fn validate_memory_julia_calibration_request_schema(schema: &Schema) -> Result<(), String> {
    validate_utf8_field(schema, MEMORY_JULIA_CALIBRATION_JOB_ID_COLUMN, false)?;
    validate_utf8_field(schema, MEMORY_JULIA_CALIBRATION_SCENARIO_PACK_COLUMN, true)?;
    validate_utf8_field(schema, MEMORY_JULIA_CALIBRATION_DATASET_REF_COLUMN, false)?;
    validate_utf8_field(schema, MEMORY_JULIA_CALIBRATION_OBJECTIVE_COLUMN, false)?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_CALIBRATION_HYPERPARAM_CONFIG_COLUMN,
        false,
    )?;
    Ok(())
}

/// Validate one staged calibration request batch.
///
/// # Errors
///
/// Returns an error when the batch violates the staged calibration semantics.
pub fn validate_memory_julia_calibration_request_batch(batch: &RecordBatch) -> Result<(), String> {
    validate_memory_julia_calibration_request_schema(batch.schema().as_ref())?;

    if batch.num_rows() == 0 {
        return Err("calibration request batch must contain at least one row".to_string());
    }

    require_non_blank_utf8_column(batch, MEMORY_JULIA_CALIBRATION_JOB_ID_COLUMN)?;
    require_non_blank_optional_utf8_column(batch, MEMORY_JULIA_CALIBRATION_SCENARIO_PACK_COLUMN)?;
    require_non_blank_utf8_column(batch, MEMORY_JULIA_CALIBRATION_DATASET_REF_COLUMN)?;
    require_non_blank_utf8_column(batch, MEMORY_JULIA_CALIBRATION_OBJECTIVE_COLUMN)?;
    require_non_blank_utf8_column(batch, MEMORY_JULIA_CALIBRATION_HYPERPARAM_CONFIG_COLUMN)?;

    Ok(())
}

/// Validate many staged calibration request batches.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any request batch violates the staged
/// contract.
pub fn validate_memory_julia_calibration_request_batches(
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    for batch in batches {
        validate_memory_julia_calibration_request_batch(batch)
            .map_err(|error| calibration_contract_error(&error))?;
    }
    Ok(())
}

/// Validate one staged calibration response schema.
///
/// # Errors
///
/// Returns an error when the schema does not match the staged calibration
/// response contract.
pub fn validate_memory_julia_calibration_response_schema(schema: &Schema) -> Result<(), String> {
    validate_utf8_field(schema, MEMORY_JULIA_CALIBRATION_JOB_ID_COLUMN, false)?;
    validate_utf8_field(schema, MEMORY_JULIA_CALIBRATION_SCENARIO_PACK_COLUMN, true)?;
    validate_utf8_field(schema, MEMORY_JULIA_CALIBRATION_ARTIFACT_REF_COLUMN, false)?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_CALIBRATION_SUMMARY_METRICS_COLUMN,
        false,
    )?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_CALIBRATION_RECOMMENDED_THRESHOLDS_COLUMN,
        false,
    )?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_CALIBRATION_RECOMMENDED_WEIGHTS_COLUMN,
        false,
    )?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_CALIBRATION_SCHEMA_VERSION_COLUMN,
        false,
    )?;
    Ok(())
}

/// Validate one staged calibration response batch.
///
/// # Errors
///
/// Returns an error when the batch violates the staged calibration response
/// semantics.
pub fn validate_memory_julia_calibration_response_batch(batch: &RecordBatch) -> Result<(), String> {
    validate_memory_julia_calibration_response_schema(batch.schema().as_ref())?;

    if batch.num_rows() == 0 {
        return Err("calibration response batch must contain at least one row".to_string());
    }

    require_non_blank_utf8_column(batch, MEMORY_JULIA_CALIBRATION_JOB_ID_COLUMN)?;
    require_non_blank_optional_utf8_column(batch, MEMORY_JULIA_CALIBRATION_SCENARIO_PACK_COLUMN)?;
    require_non_blank_utf8_column(batch, MEMORY_JULIA_CALIBRATION_ARTIFACT_REF_COLUMN)?;
    require_non_blank_utf8_column(batch, MEMORY_JULIA_CALIBRATION_SUMMARY_METRICS_COLUMN)?;
    require_non_blank_utf8_column(
        batch,
        MEMORY_JULIA_CALIBRATION_RECOMMENDED_THRESHOLDS_COLUMN,
    )?;
    require_non_blank_utf8_column(batch, MEMORY_JULIA_CALIBRATION_RECOMMENDED_WEIGHTS_COLUMN)?;
    require_non_blank_utf8_column(batch, MEMORY_JULIA_CALIBRATION_SCHEMA_VERSION_COLUMN)?;

    Ok(())
}

/// Validate many staged calibration response batches.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any response batch violates the
/// staged contract.
pub fn validate_memory_julia_calibration_response_batches(
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    for batch in batches {
        validate_memory_julia_calibration_response_batch(batch)
            .map_err(|error| calibration_contract_error(&error))?;
    }
    Ok(())
}

/// Decode many staged calibration response batches into typed artifact rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any batch violates the staged
/// response contract.
pub fn decode_memory_julia_calibration_artifact_rows(
    batches: &[RecordBatch],
) -> Result<Vec<MemoryJuliaCalibrationArtifactRow>, RepoIntelligenceError> {
    validate_memory_julia_calibration_response_batches(batches)?;

    let mut rows = Vec::new();
    for batch in batches {
        let calibration_job_id = utf8_column(batch, MEMORY_JULIA_CALIBRATION_JOB_ID_COLUMN)?;
        let scenario_pack = utf8_column(batch, MEMORY_JULIA_CALIBRATION_SCENARIO_PACK_COLUMN)?;
        let artifact_ref = utf8_column(batch, MEMORY_JULIA_CALIBRATION_ARTIFACT_REF_COLUMN)?;
        let summary_metrics = utf8_column(batch, MEMORY_JULIA_CALIBRATION_SUMMARY_METRICS_COLUMN)?;
        let recommended_thresholds = utf8_column(
            batch,
            MEMORY_JULIA_CALIBRATION_RECOMMENDED_THRESHOLDS_COLUMN,
        )?;
        let recommended_weights =
            utf8_column(batch, MEMORY_JULIA_CALIBRATION_RECOMMENDED_WEIGHTS_COLUMN)?;
        let schema_version = utf8_column(batch, MEMORY_JULIA_CALIBRATION_SCHEMA_VERSION_COLUMN)?;

        for row in 0..batch.num_rows() {
            rows.push(MemoryJuliaCalibrationArtifactRow {
                calibration_job_id: calibration_job_id.value(row).to_string(),
                scenario_pack: (!scenario_pack.is_null(row))
                    .then(|| scenario_pack.value(row).to_string()),
                artifact_ref: artifact_ref.value(row).to_string(),
                summary_metrics: summary_metrics.value(row).to_string(),
                recommended_thresholds: recommended_thresholds.value(row).to_string(),
                recommended_weights: recommended_weights.value(row).to_string(),
                schema_version: schema_version.value(row).to_string(),
            });
        }
    }

    Ok(rows)
}

fn validate_utf8_field(schema: &Schema, name: &str, nullable: bool) -> Result<(), String> {
    let field = schema
        .field_with_name(name)
        .map_err(|_| format!("missing `{name}` field"))?;
    if field.data_type() != &DataType::Utf8 {
        return Err(format!(
            "`{name}` must use {:?}, found {:?}",
            DataType::Utf8,
            field.data_type()
        ));
    }
    if field.is_nullable() != nullable {
        return Err(format!("`{name}` nullable mismatch"));
    }
    Ok(())
}

fn utf8_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
) -> Result<&'a StringArray, RepoIntelligenceError> {
    batch
        .column_by_name(name)
        .ok_or_else(|| calibration_contract_error(&format!("missing `{name}` column")))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| calibration_contract_error(&format!("`{name}` must be Utf8")))
}

fn require_non_blank_utf8_column(batch: &RecordBatch, name: &str) -> Result<(), String> {
    let column = batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("`{name}` must be Utf8"))?;

    for row in 0..batch.num_rows() {
        if column.is_null(row) || column.value(row).trim().is_empty() {
            return Err(format!("`{name}` contains a blank value at row {row}"));
        }
    }
    Ok(())
}

fn require_non_blank_optional_utf8_column(batch: &RecordBatch, name: &str) -> Result<(), String> {
    let column = batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("`{name}` must be Utf8"))?;

    for row in 0..batch.num_rows() {
        if !column.is_null(row) && column.value(row).trim().is_empty() {
            return Err(format!("`{name}` contains a blank value at row {row}"));
        }
    }
    Ok(())
}

fn calibration_contract_error(message: &str) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!("memory Julia memory_calibration contract violation: {message}"),
    }
}
