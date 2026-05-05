//! Julia recall-plan tuning transport contract and row processing.

use std::sync::Arc;

use arrow::array::{Array, Float32Array, StringArray, UInt32Array, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

/// Request column carrying the logical scope.
pub const MEMORY_JULIA_PLAN_TUNING_SCOPE_COLUMN: &str = "scope";
/// Request column carrying the optional scenario pack.
pub const MEMORY_JULIA_PLAN_TUNING_SCENARIO_PACK_COLUMN: &str = "scenario_pack";
/// Request column carrying the current `k1`.
pub const MEMORY_JULIA_PLAN_TUNING_CURRENT_K1_COLUMN: &str = "current_k1";
/// Request column carrying the current `k2`.
pub const MEMORY_JULIA_PLAN_TUNING_CURRENT_K2_COLUMN: &str = "current_k2";
/// Request column carrying the current `lambda`.
pub const MEMORY_JULIA_PLAN_TUNING_CURRENT_LAMBDA_COLUMN: &str = "current_lambda";
/// Request column carrying the current `min_score`.
pub const MEMORY_JULIA_PLAN_TUNING_CURRENT_MIN_SCORE_COLUMN: &str = "current_min_score";
/// Request column carrying the current max-context budget.
pub const MEMORY_JULIA_PLAN_TUNING_CURRENT_MAX_CONTEXT_CHARS_COLUMN: &str =
    "current_max_context_chars";
/// Request column carrying the normalized feedback bias.
pub const MEMORY_JULIA_PLAN_TUNING_FEEDBACK_BIAS_COLUMN: &str = "feedback_bias";
/// Request column carrying the recent success rate.
pub const MEMORY_JULIA_PLAN_TUNING_RECENT_SUCCESS_RATE_COLUMN: &str = "recent_success_rate";
/// Request column carrying the recent failure rate.
pub const MEMORY_JULIA_PLAN_TUNING_RECENT_FAILURE_RATE_COLUMN: &str = "recent_failure_rate";
/// Request column carrying the recent latency budget.
pub const MEMORY_JULIA_PLAN_TUNING_RECENT_LATENCY_MS_COLUMN: &str = "recent_latency_ms";

/// Response column carrying the next `k1`.
pub const MEMORY_JULIA_PLAN_TUNING_NEXT_K1_COLUMN: &str = "next_k1";
/// Response column carrying the next `k2`.
pub const MEMORY_JULIA_PLAN_TUNING_NEXT_K2_COLUMN: &str = "next_k2";
/// Response column carrying the next `lambda`.
pub const MEMORY_JULIA_PLAN_TUNING_NEXT_LAMBDA_COLUMN: &str = "next_lambda";
/// Response column carrying the next `min_score`.
pub const MEMORY_JULIA_PLAN_TUNING_NEXT_MIN_SCORE_COLUMN: &str = "next_min_score";
/// Response column carrying the next max-context budget.
pub const MEMORY_JULIA_PLAN_TUNING_NEXT_MAX_CONTEXT_CHARS_COLUMN: &str = "next_max_context_chars";
/// Response column carrying the tuning rationale.
pub const MEMORY_JULIA_PLAN_TUNING_REASON_COLUMN: &str = "tuning_reason";
/// Response column carrying the confidence score.
pub const MEMORY_JULIA_PLAN_TUNING_CONFIDENCE_COLUMN: &str = "confidence";
/// Response column carrying the physical schema version echoed by the provider.
pub const MEMORY_JULIA_PLAN_TUNING_SCHEMA_VERSION_COLUMN: &str = "schema_version";

/// Canonical request column order for the staged plan-tuning profile.
pub const MEMORY_JULIA_PLAN_TUNING_REQUEST_COLUMNS: [&str; 11] = [
    MEMORY_JULIA_PLAN_TUNING_SCOPE_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_SCENARIO_PACK_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_CURRENT_K1_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_CURRENT_K2_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_CURRENT_LAMBDA_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_CURRENT_MIN_SCORE_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_CURRENT_MAX_CONTEXT_CHARS_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_FEEDBACK_BIAS_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_RECENT_SUCCESS_RATE_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_RECENT_FAILURE_RATE_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_RECENT_LATENCY_MS_COLUMN,
];

/// Canonical response column order for the staged plan-tuning profile.
pub const MEMORY_JULIA_PLAN_TUNING_RESPONSE_COLUMNS: [&str; 9] = [
    MEMORY_JULIA_PLAN_TUNING_SCOPE_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_NEXT_K1_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_NEXT_K2_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_NEXT_LAMBDA_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_NEXT_MIN_SCORE_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_NEXT_MAX_CONTEXT_CHARS_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_REASON_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_CONFIDENCE_COLUMN,
    MEMORY_JULIA_PLAN_TUNING_SCHEMA_VERSION_COLUMN,
];

/// One typed request row for the staged plan-tuning profile.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryJuliaPlanTuningRequestRow {
    /// Logical scope of the tuning context.
    pub scope: String,
    /// Optional scenario pack forwarded into Julia.
    pub scenario_pack: Option<String>,
    /// Current phase-1 candidate count.
    pub current_k1: u32,
    /// Current phase-2 rerank output count.
    pub current_k2: u32,
    /// Current blending weight.
    pub current_lambda: f32,
    /// Current minimum retained similarity score.
    pub current_min_score: f32,
    /// Current max-context budget in chars.
    pub current_max_context_chars: u32,
    /// Normalized feedback bias in `[-1, 1]`.
    pub feedback_bias: f32,
    /// Recent success rate in `[0, 1]`.
    pub recent_success_rate: f32,
    /// Recent failure rate in `[0, 1]`.
    pub recent_failure_rate: f32,
    /// Recent latency in milliseconds.
    pub recent_latency_ms: u64,
}

/// One decoded advice row from the staged plan-tuning profile.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryJuliaPlanTuningAdviceRow {
    /// Scope echoed by the provider.
    pub scope: String,
    /// Recommended next phase-1 candidate count.
    pub next_k1: u32,
    /// Recommended next phase-2 rerank output count.
    pub next_k2: u32,
    /// Recommended next blending weight.
    pub next_lambda: f32,
    /// Recommended next minimum retained similarity score.
    pub next_min_score: f32,
    /// Recommended next max-context budget.
    pub next_max_context_chars: u32,
    /// Human-readable tuning rationale.
    pub tuning_reason: String,
    /// Confidence score in `[0, 1]`.
    pub confidence: f32,
    /// Physical schema version echoed by the provider.
    pub schema_version: String,
}

/// Build the staged plan-tuning request schema.
#[must_use]
pub fn memory_julia_plan_tuning_request_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new(MEMORY_JULIA_PLAN_TUNING_SCOPE_COLUMN, DataType::Utf8, false),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_SCENARIO_PACK_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_CURRENT_K1_COLUMN,
            DataType::UInt32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_CURRENT_K2_COLUMN,
            DataType::UInt32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_CURRENT_LAMBDA_COLUMN,
            DataType::Float32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_CURRENT_MIN_SCORE_COLUMN,
            DataType::Float32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_CURRENT_MAX_CONTEXT_CHARS_COLUMN,
            DataType::UInt32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_FEEDBACK_BIAS_COLUMN,
            DataType::Float32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_RECENT_SUCCESS_RATE_COLUMN,
            DataType::Float32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_RECENT_FAILURE_RATE_COLUMN,
            DataType::Float32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_RECENT_LATENCY_MS_COLUMN,
            DataType::UInt64,
            false,
        ),
    ]))
}

/// Build the staged plan-tuning response schema.
#[must_use]
pub fn memory_julia_plan_tuning_response_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new(MEMORY_JULIA_PLAN_TUNING_SCOPE_COLUMN, DataType::Utf8, false),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_NEXT_K1_COLUMN,
            DataType::UInt32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_NEXT_K2_COLUMN,
            DataType::UInt32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_NEXT_LAMBDA_COLUMN,
            DataType::Float32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_NEXT_MIN_SCORE_COLUMN,
            DataType::Float32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_NEXT_MAX_CONTEXT_CHARS_COLUMN,
            DataType::UInt32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_REASON_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_CONFIDENCE_COLUMN,
            DataType::Float32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_PLAN_TUNING_SCHEMA_VERSION_COLUMN,
            DataType::Utf8,
            false,
        ),
    ]))
}

/// Build one staged plan-tuning request batch from typed rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the request rows violate the staged
/// plan-tuning contract.
pub fn build_memory_julia_plan_tuning_request_batch(
    rows: &[MemoryJuliaPlanTuningRequestRow],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let batch = RecordBatch::try_new(
        memory_julia_plan_tuning_request_schema(),
        vec![
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.scope.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.scenario_pack.as_deref())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter().map(|row| row.current_k1).collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter().map(|row| row.current_k2).collect::<Vec<_>>(),
            )),
            Arc::new(Float32Array::from(
                rows.iter()
                    .map(|row| row.current_lambda)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Float32Array::from(
                rows.iter()
                    .map(|row| row.current_min_score)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter()
                    .map(|row| row.current_max_context_chars)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Float32Array::from(
                rows.iter().map(|row| row.feedback_bias).collect::<Vec<_>>(),
            )),
            Arc::new(Float32Array::from(
                rows.iter()
                    .map(|row| row.recent_success_rate)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Float32Array::from(
                rows.iter()
                    .map(|row| row.recent_failure_rate)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter()
                    .map(|row| row.recent_latency_ms)
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(|error| plan_tuning_contract_error(&error.to_string()))?;

    validate_memory_julia_plan_tuning_request_batch(&batch)
        .map_err(|error| plan_tuning_contract_error(&error))?;
    Ok(batch)
}

/// Validate one staged plan-tuning request schema.
///
/// # Errors
///
/// Returns an error when the schema does not match the staged plan-tuning
/// contract.
pub fn validate_memory_julia_plan_tuning_request_schema(schema: &Schema) -> Result<(), String> {
    validate_utf8_field(schema, MEMORY_JULIA_PLAN_TUNING_SCOPE_COLUMN, false)?;
    validate_utf8_field(schema, MEMORY_JULIA_PLAN_TUNING_SCENARIO_PACK_COLUMN, true)?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_PLAN_TUNING_CURRENT_K1_COLUMN,
        &DataType::UInt32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_PLAN_TUNING_CURRENT_K2_COLUMN,
        &DataType::UInt32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_PLAN_TUNING_CURRENT_LAMBDA_COLUMN,
        &DataType::Float32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_PLAN_TUNING_CURRENT_MIN_SCORE_COLUMN,
        &DataType::Float32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_PLAN_TUNING_CURRENT_MAX_CONTEXT_CHARS_COLUMN,
        &DataType::UInt32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_PLAN_TUNING_FEEDBACK_BIAS_COLUMN,
        &DataType::Float32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_PLAN_TUNING_RECENT_SUCCESS_RATE_COLUMN,
        &DataType::Float32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_PLAN_TUNING_RECENT_FAILURE_RATE_COLUMN,
        &DataType::Float32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_PLAN_TUNING_RECENT_LATENCY_MS_COLUMN,
        &DataType::UInt64,
        false,
    )?;
    Ok(())
}

/// Validate one staged plan-tuning request batch.
///
/// # Errors
///
/// Returns an error when the batch violates the staged plan-tuning semantics.
pub fn validate_memory_julia_plan_tuning_request_batch(batch: &RecordBatch) -> Result<(), String> {
    validate_memory_julia_plan_tuning_request_schema(batch.schema().as_ref())?;

    if batch.num_rows() == 0 {
        return Err("plan tuning request batch must contain at least one row".to_string());
    }

    require_non_blank_utf8_column(batch, MEMORY_JULIA_PLAN_TUNING_SCOPE_COLUMN)?;
    require_non_blank_optional_utf8_column(batch, MEMORY_JULIA_PLAN_TUNING_SCENARIO_PACK_COLUMN)?;
    require_positive_u32_column(batch, MEMORY_JULIA_PLAN_TUNING_CURRENT_K1_COLUMN)?;
    require_positive_u32_column(batch, MEMORY_JULIA_PLAN_TUNING_CURRENT_K2_COLUMN)?;
    require_probability_column(batch, MEMORY_JULIA_PLAN_TUNING_CURRENT_LAMBDA_COLUMN)?;
    require_probability_column(batch, MEMORY_JULIA_PLAN_TUNING_CURRENT_MIN_SCORE_COLUMN)?;
    require_positive_u32_column(
        batch,
        MEMORY_JULIA_PLAN_TUNING_CURRENT_MAX_CONTEXT_CHARS_COLUMN,
    )?;
    require_signed_unit_column(batch, MEMORY_JULIA_PLAN_TUNING_FEEDBACK_BIAS_COLUMN)?;
    require_probability_column(batch, MEMORY_JULIA_PLAN_TUNING_RECENT_SUCCESS_RATE_COLUMN)?;
    require_probability_column(batch, MEMORY_JULIA_PLAN_TUNING_RECENT_FAILURE_RATE_COLUMN)?;

    let current_k1 = u32_column(batch, MEMORY_JULIA_PLAN_TUNING_CURRENT_K1_COLUMN)
        .map_err(|error| error.to_string())?;
    let current_k2 = u32_column(batch, MEMORY_JULIA_PLAN_TUNING_CURRENT_K2_COLUMN)
        .map_err(|error| error.to_string())?;
    let success_rate = float32_column(batch, MEMORY_JULIA_PLAN_TUNING_RECENT_SUCCESS_RATE_COLUMN)
        .map_err(|error| error.to_string())?;
    let failure_rate = float32_column(batch, MEMORY_JULIA_PLAN_TUNING_RECENT_FAILURE_RATE_COLUMN)
        .map_err(|error| error.to_string())?;

    for row in 0..batch.num_rows() {
        if current_k2.value(row) > current_k1.value(row) {
            return Err(format!(
                "plan tuning request row {row} has current_k2 greater than current_k1"
            ));
        }

        let combined_rate = success_rate.value(row) + failure_rate.value(row);
        if combined_rate > 1.0 + f32::EPSILON {
            return Err(format!(
                "plan tuning request row {row} has recent success/failure rates summing above 1.0"
            ));
        }
    }

    Ok(())
}

/// Validate many staged plan-tuning request batches.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any request batch violates the staged
/// contract.
pub fn validate_memory_julia_plan_tuning_request_batches(
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    for batch in batches {
        validate_memory_julia_plan_tuning_request_batch(batch)
            .map_err(|error| plan_tuning_contract_error(&error))?;
    }
    Ok(())
}

/// Validate one staged plan-tuning response schema.
///
/// # Errors
///
/// Returns an error when the schema does not match the staged plan-tuning
/// response contract.
pub fn validate_memory_julia_plan_tuning_response_schema(schema: &Schema) -> Result<(), String> {
    validate_utf8_field(schema, MEMORY_JULIA_PLAN_TUNING_SCOPE_COLUMN, false)?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_PLAN_TUNING_NEXT_K1_COLUMN,
        &DataType::UInt32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_PLAN_TUNING_NEXT_K2_COLUMN,
        &DataType::UInt32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_PLAN_TUNING_NEXT_LAMBDA_COLUMN,
        &DataType::Float32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_PLAN_TUNING_NEXT_MIN_SCORE_COLUMN,
        &DataType::Float32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_PLAN_TUNING_NEXT_MAX_CONTEXT_CHARS_COLUMN,
        &DataType::UInt32,
        false,
    )?;
    validate_utf8_field(schema, MEMORY_JULIA_PLAN_TUNING_REASON_COLUMN, false)?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_PLAN_TUNING_CONFIDENCE_COLUMN,
        &DataType::Float32,
        false,
    )?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_PLAN_TUNING_SCHEMA_VERSION_COLUMN,
        false,
    )?;
    Ok(())
}

/// Validate one staged plan-tuning response batch.
///
/// # Errors
///
/// Returns an error when the batch violates the staged plan-tuning response
/// semantics.
pub fn validate_memory_julia_plan_tuning_response_batch(batch: &RecordBatch) -> Result<(), String> {
    validate_memory_julia_plan_tuning_response_schema(batch.schema().as_ref())?;

    if batch.num_rows() == 0 {
        return Err("plan tuning response batch must contain at least one row".to_string());
    }

    require_non_blank_utf8_column(batch, MEMORY_JULIA_PLAN_TUNING_SCOPE_COLUMN)?;
    require_positive_u32_column(batch, MEMORY_JULIA_PLAN_TUNING_NEXT_K1_COLUMN)?;
    require_positive_u32_column(batch, MEMORY_JULIA_PLAN_TUNING_NEXT_K2_COLUMN)?;
    require_probability_column(batch, MEMORY_JULIA_PLAN_TUNING_NEXT_LAMBDA_COLUMN)?;
    require_probability_column(batch, MEMORY_JULIA_PLAN_TUNING_NEXT_MIN_SCORE_COLUMN)?;
    require_positive_u32_column(
        batch,
        MEMORY_JULIA_PLAN_TUNING_NEXT_MAX_CONTEXT_CHARS_COLUMN,
    )?;
    require_non_blank_utf8_column(batch, MEMORY_JULIA_PLAN_TUNING_REASON_COLUMN)?;
    require_probability_column(batch, MEMORY_JULIA_PLAN_TUNING_CONFIDENCE_COLUMN)?;
    require_non_blank_utf8_column(batch, MEMORY_JULIA_PLAN_TUNING_SCHEMA_VERSION_COLUMN)?;

    let next_k1 = u32_column(batch, MEMORY_JULIA_PLAN_TUNING_NEXT_K1_COLUMN)
        .map_err(|error| error.to_string())?;
    let next_k2 = u32_column(batch, MEMORY_JULIA_PLAN_TUNING_NEXT_K2_COLUMN)
        .map_err(|error| error.to_string())?;

    for row in 0..batch.num_rows() {
        if next_k2.value(row) > next_k1.value(row) {
            return Err(format!(
                "plan tuning response row {row} has next_k2 greater than next_k1"
            ));
        }
    }

    Ok(())
}

/// Validate many staged plan-tuning response batches.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any response batch violates the
/// staged contract.
pub fn validate_memory_julia_plan_tuning_response_batches(
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    for batch in batches {
        validate_memory_julia_plan_tuning_response_batch(batch)
            .map_err(|error| plan_tuning_contract_error(&error))?;
    }
    Ok(())
}

/// Decode many staged plan-tuning response batches into typed advice rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any batch violates the staged
/// response contract.
pub fn decode_memory_julia_plan_tuning_advice_rows(
    batches: &[RecordBatch],
) -> Result<Vec<MemoryJuliaPlanTuningAdviceRow>, RepoIntelligenceError> {
    validate_memory_julia_plan_tuning_response_batches(batches)?;

    let mut rows = Vec::new();
    for batch in batches {
        let scope = utf8_column(batch, MEMORY_JULIA_PLAN_TUNING_SCOPE_COLUMN)?;
        let next_k1 = u32_column(batch, MEMORY_JULIA_PLAN_TUNING_NEXT_K1_COLUMN)?;
        let next_k2 = u32_column(batch, MEMORY_JULIA_PLAN_TUNING_NEXT_K2_COLUMN)?;
        let next_lambda = float32_column(batch, MEMORY_JULIA_PLAN_TUNING_NEXT_LAMBDA_COLUMN)?;
        let next_min_score = float32_column(batch, MEMORY_JULIA_PLAN_TUNING_NEXT_MIN_SCORE_COLUMN)?;
        let next_max_context_chars = u32_column(
            batch,
            MEMORY_JULIA_PLAN_TUNING_NEXT_MAX_CONTEXT_CHARS_COLUMN,
        )?;
        let tuning_reason = utf8_column(batch, MEMORY_JULIA_PLAN_TUNING_REASON_COLUMN)?;
        let confidence = float32_column(batch, MEMORY_JULIA_PLAN_TUNING_CONFIDENCE_COLUMN)?;
        let schema_version = utf8_column(batch, MEMORY_JULIA_PLAN_TUNING_SCHEMA_VERSION_COLUMN)?;

        for row in 0..batch.num_rows() {
            rows.push(MemoryJuliaPlanTuningAdviceRow {
                scope: scope.value(row).to_string(),
                next_k1: next_k1.value(row),
                next_k2: next_k2.value(row),
                next_lambda: next_lambda.value(row),
                next_min_score: next_min_score.value(row),
                next_max_context_chars: next_max_context_chars.value(row),
                tuning_reason: tuning_reason.value(row).to_string(),
                confidence: confidence.value(row),
                schema_version: schema_version.value(row).to_string(),
            });
        }
    }

    Ok(rows)
}

fn validate_utf8_field(schema: &Schema, name: &str, nullable: bool) -> Result<(), String> {
    validate_primitive_field(schema, name, &DataType::Utf8, nullable)
}

fn validate_primitive_field(
    schema: &Schema,
    name: &str,
    data_type: &DataType,
    nullable: bool,
) -> Result<(), String> {
    let field = schema
        .field_with_name(name)
        .map_err(|_| format!("missing `{name}` field"))?;
    if field.data_type() != data_type {
        return Err(format!(
            "`{name}` must use {:?}, found {:?}",
            data_type,
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
        .ok_or_else(|| plan_tuning_contract_error(&format!("missing `{name}` column")))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| plan_tuning_contract_error(&format!("`{name}` must be Utf8")))
}

fn float32_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
) -> Result<&'a Float32Array, RepoIntelligenceError> {
    batch
        .column_by_name(name)
        .ok_or_else(|| plan_tuning_contract_error(&format!("missing `{name}` column")))?
        .as_any()
        .downcast_ref::<Float32Array>()
        .ok_or_else(|| plan_tuning_contract_error(&format!("`{name}` must be Float32")))
}

fn u32_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
) -> Result<&'a UInt32Array, RepoIntelligenceError> {
    batch
        .column_by_name(name)
        .ok_or_else(|| plan_tuning_contract_error(&format!("missing `{name}` column")))?
        .as_any()
        .downcast_ref::<UInt32Array>()
        .ok_or_else(|| plan_tuning_contract_error(&format!("`{name}` must be UInt32")))
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

fn require_probability_column(batch: &RecordBatch, name: &str) -> Result<(), String> {
    let column = batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<Float32Array>()
        .ok_or_else(|| format!("`{name}` must be Float32"))?;

    for row in 0..batch.num_rows() {
        let value = column.value(row);
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(format!(
                "`{name}` must contain finite values in [0, 1]; found {value} at row {row}"
            ));
        }
    }
    Ok(())
}

fn require_signed_unit_column(batch: &RecordBatch, name: &str) -> Result<(), String> {
    let column = batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<Float32Array>()
        .ok_or_else(|| format!("`{name}` must be Float32"))?;

    for row in 0..batch.num_rows() {
        let value = column.value(row);
        if !value.is_finite() || !(-1.0..=1.0).contains(&value) {
            return Err(format!(
                "`{name}` must contain finite values in [-1, 1]; found {value} at row {row}"
            ));
        }
    }
    Ok(())
}

fn require_positive_u32_column(batch: &RecordBatch, name: &str) -> Result<(), String> {
    let column = batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<UInt32Array>()
        .ok_or_else(|| format!("`{name}` must be UInt32"))?;

    for row in 0..batch.num_rows() {
        if column.value(row) == 0 {
            return Err(format!("`{name}` must be greater than zero at row {row}"));
        }
    }
    Ok(())
}

fn plan_tuning_contract_error(message: &str) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!("memory Julia memory_plan_tuning contract violation: {message}"),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/memory/plan_tuning.rs"]
mod tests;
