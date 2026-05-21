//! Schema, builders, validation, and decoding for Julia episodic recall.

use std::sync::Arc;

use arrow::array::{
    Array, Float32Array, Float32Builder, Int64Array, ListArray, ListBuilder, StringArray,
    UInt32Array,
};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

#[cfg(test)]
use super::MEMORY_JULIA_EPISODIC_RECALL_REQUEST_COLUMNS;
use super::{
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
    MEMORY_JULIA_EPISODIC_RECALL_RETRIEVAL_COUNT_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_RETRIEVAL_MODE_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_SCENARIO_PACK_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_SCHEMA_VERSION_COLUMN, MEMORY_JULIA_EPISODIC_RECALL_SCOPE_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_SEMANTIC_SCORE_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_SUCCESS_COUNT_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_UPDATED_AT_MS_COLUMN,
    MEMORY_JULIA_EPISODIC_RECALL_UTILITY_SCORE_COLUMN, MemoryJuliaEpisodicRecallRequestRow,
    MemoryJuliaEpisodicRecallScoreRow,
};

/// Build the staged episodic recall request schema.
#[must_use]
pub fn memory_julia_episodic_recall_request_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_QUERY_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_SCENARIO_PACK_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_SCOPE_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_QUERY_TEXT_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_QUERY_EMBEDDING_COLUMN,
            DataType::List(Arc::new(Field::new("item", DataType::Float32, true))),
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_CANDIDATE_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_INTENT_EMBEDDING_COLUMN,
            DataType::List(Arc::new(Field::new("item", DataType::Float32, true))),
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_Q_VALUE_COLUMN,
            DataType::Float32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_SUCCESS_COUNT_COLUMN,
            DataType::UInt32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_FAILURE_COUNT_COLUMN,
            DataType::UInt32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_RETRIEVAL_COUNT_COLUMN,
            DataType::UInt32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_CREATED_AT_MS_COLUMN,
            DataType::Int64,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_UPDATED_AT_MS_COLUMN,
            DataType::Int64,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_K1_COLUMN,
            DataType::Float32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_K2_COLUMN,
            DataType::Float32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_LAMBDA_COLUMN,
            DataType::Float32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_MIN_SCORE_COLUMN,
            DataType::Float32,
            false,
        ),
    ]))
}

/// Build the staged episodic recall response schema.
#[must_use]
pub fn memory_julia_episodic_recall_response_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_QUERY_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_CANDIDATE_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_SEMANTIC_SCORE_COLUMN,
            DataType::Float32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_UTILITY_SCORE_COLUMN,
            DataType::Float32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_FINAL_SCORE_COLUMN,
            DataType::Float32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_CONFIDENCE_COLUMN,
            DataType::Float32,
            false,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_RANKING_REASON_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_RETRIEVAL_MODE_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            MEMORY_JULIA_EPISODIC_RECALL_SCHEMA_VERSION_COLUMN,
            DataType::Utf8,
            false,
        ),
    ]))
}

/// Build one staged episodic recall request batch from typed rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the request rows cannot be
/// materialized or violate the staged contract.
pub fn build_memory_julia_episodic_recall_request_batch(
    rows: &[MemoryJuliaEpisodicRecallRequestRow],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let batch = RecordBatch::try_new(
        memory_julia_episodic_recall_request_schema(),
        vec![
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.query_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.scenario_pack.as_deref())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.scope.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.query_text.as_deref())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(build_float32_list_array(
                rows.iter().map(|row| row.query_embedding.as_slice()),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.candidate_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(build_float32_list_array(
                rows.iter().map(|row| row.intent_embedding.as_slice()),
            )),
            Arc::new(Float32Array::from(
                rows.iter().map(|row| row.q_value).collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter().map(|row| row.success_count).collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter().map(|row| row.failure_count).collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter()
                    .map(|row| row.retrieval_count)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Int64Array::from(
                rows.iter()
                    .map(|row| row.created_at_ms.value())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Int64Array::from(
                rows.iter()
                    .map(|row| row.updated_at_ms.value())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Float32Array::from(
                rows.iter().map(|row| row.k1).collect::<Vec<_>>(),
            )),
            Arc::new(Float32Array::from(
                rows.iter().map(|row| row.k2).collect::<Vec<_>>(),
            )),
            Arc::new(Float32Array::from(
                rows.iter().map(|row| row.lambda).collect::<Vec<_>>(),
            )),
            Arc::new(Float32Array::from(
                rows.iter().map(|row| row.min_score).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(|error| episodic_recall_contract_error(&error.to_string()))?;

    validate_memory_julia_episodic_recall_request_batch(&batch)
        .map_err(|error| episodic_recall_contract_error(&error))?;
    Ok(batch)
}

/// Validate one staged episodic recall request schema.
///
/// # Errors
///
/// Returns an error when the schema does not match the staged contract.
pub fn validate_memory_julia_episodic_recall_request_schema(schema: &Schema) -> Result<(), String> {
    validate_episodic_recall_request_text_schema(schema)?;
    validate_episodic_recall_request_embedding_schema(schema)?;
    validate_episodic_recall_request_history_schema(schema)?;
    validate_episodic_recall_request_tuning_schema(schema)
}

fn validate_episodic_recall_request_text_schema(schema: &Schema) -> Result<(), String> {
    validate_utf8_field(schema, MEMORY_JULIA_EPISODIC_RECALL_QUERY_ID_COLUMN, false)?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_SCENARIO_PACK_COLUMN,
        true,
    )?;
    validate_utf8_field(schema, MEMORY_JULIA_EPISODIC_RECALL_SCOPE_COLUMN, false)?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_CANDIDATE_ID_COLUMN,
        false,
    )
}

fn validate_episodic_recall_request_embedding_schema(schema: &Schema) -> Result<(), String> {
    validate_utf8_field(schema, MEMORY_JULIA_EPISODIC_RECALL_QUERY_TEXT_COLUMN, true)?;
    validate_float32_list_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_QUERY_EMBEDDING_COLUMN,
        false,
    )?;
    validate_float32_list_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_INTENT_EMBEDDING_COLUMN,
        false,
    )
}

fn validate_episodic_recall_request_history_schema(schema: &Schema) -> Result<(), String> {
    validate_primitive_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_Q_VALUE_COLUMN,
        &DataType::Float32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_SUCCESS_COUNT_COLUMN,
        &DataType::UInt32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_FAILURE_COUNT_COLUMN,
        &DataType::UInt32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_RETRIEVAL_COUNT_COLUMN,
        &DataType::UInt32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_CREATED_AT_MS_COLUMN,
        &DataType::Int64,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_UPDATED_AT_MS_COLUMN,
        &DataType::Int64,
        false,
    )
}

fn validate_episodic_recall_request_tuning_schema(schema: &Schema) -> Result<(), String> {
    validate_primitive_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_K1_COLUMN,
        &DataType::Float32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_K2_COLUMN,
        &DataType::Float32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_LAMBDA_COLUMN,
        &DataType::Float32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_MIN_SCORE_COLUMN,
        &DataType::Float32,
        false,
    )
}

/// Validate one staged episodic recall request batch.
///
/// # Errors
///
/// Returns an error when the batch violates the staged contract semantics.
pub fn validate_memory_julia_episodic_recall_request_batch(
    batch: &RecordBatch,
) -> Result<(), String> {
    validate_memory_julia_episodic_recall_request_schema(batch.schema().as_ref())?;

    if batch.num_rows() == 0 {
        return Err("episodic recall request batch must contain at least one row".to_string());
    }

    validate_episodic_recall_request_text_columns(batch)?;
    validate_episodic_recall_request_score_columns(batch)?;
    validate_episodic_recall_request_timestamp_columns(batch)
}

fn validate_episodic_recall_request_text_columns(batch: &RecordBatch) -> Result<(), String> {
    require_non_blank_utf8_column(batch, MEMORY_JULIA_EPISODIC_RECALL_QUERY_ID_COLUMN)?;
    require_non_blank_optional_utf8_column(
        batch,
        MEMORY_JULIA_EPISODIC_RECALL_SCENARIO_PACK_COLUMN,
    )?;
    require_non_blank_utf8_column(batch, MEMORY_JULIA_EPISODIC_RECALL_SCOPE_COLUMN)?;
    require_non_blank_optional_utf8_column(batch, MEMORY_JULIA_EPISODIC_RECALL_QUERY_TEXT_COLUMN)?;
    require_non_empty_float32_list_column(
        batch,
        MEMORY_JULIA_EPISODIC_RECALL_QUERY_EMBEDDING_COLUMN,
    )?;
    require_non_blank_utf8_column(batch, MEMORY_JULIA_EPISODIC_RECALL_CANDIDATE_ID_COLUMN)?;
    require_non_empty_float32_list_column(
        batch,
        MEMORY_JULIA_EPISODIC_RECALL_INTENT_EMBEDDING_COLUMN,
    )
}

fn validate_episodic_recall_request_score_columns(batch: &RecordBatch) -> Result<(), String> {
    require_finite_float32_column(batch, MEMORY_JULIA_EPISODIC_RECALL_Q_VALUE_COLUMN)?;
    require_finite_non_negative_float32_column(batch, MEMORY_JULIA_EPISODIC_RECALL_K1_COLUMN)?;
    require_finite_non_negative_float32_column(batch, MEMORY_JULIA_EPISODIC_RECALL_K2_COLUMN)?;
    require_finite_non_negative_float32_column(batch, MEMORY_JULIA_EPISODIC_RECALL_LAMBDA_COLUMN)?;
    require_finite_non_negative_float32_column(batch, MEMORY_JULIA_EPISODIC_RECALL_MIN_SCORE_COLUMN)
}

fn validate_episodic_recall_request_timestamp_columns(batch: &RecordBatch) -> Result<(), String> {
    let created_at = int64_column(batch, MEMORY_JULIA_EPISODIC_RECALL_CREATED_AT_MS_COLUMN)
        .map_err(|error| error.to_string())?;
    let updated_at = int64_column(batch, MEMORY_JULIA_EPISODIC_RECALL_UPDATED_AT_MS_COLUMN)
        .map_err(|error| error.to_string())?;
    for row in 0..batch.num_rows() {
        if updated_at.value(row) < created_at.value(row) {
            return Err(format!(
                "episodic recall request row {row} has updated_at_ms earlier than created_at_ms"
            ));
        }
    }

    Ok(())
}

/// Validate many staged episodic recall request batches.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any request batch violates the staged
/// contract.
pub fn validate_memory_julia_episodic_recall_request_batches(
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    for batch in batches {
        validate_memory_julia_episodic_recall_request_batch(batch)
            .map_err(|error| episodic_recall_contract_error(&error))?;
    }
    Ok(())
}

/// Validate the staged episodic recall response schema.
///
/// # Errors
///
/// Returns an error when the schema does not match the staged response contract.
pub fn validate_memory_julia_episodic_recall_response_schema(
    schema: &Schema,
) -> Result<(), String> {
    validate_utf8_field(schema, MEMORY_JULIA_EPISODIC_RECALL_QUERY_ID_COLUMN, false)?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_CANDIDATE_ID_COLUMN,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_SEMANTIC_SCORE_COLUMN,
        &DataType::Float32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_UTILITY_SCORE_COLUMN,
        &DataType::Float32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_FINAL_SCORE_COLUMN,
        &DataType::Float32,
        false,
    )?;
    validate_primitive_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_CONFIDENCE_COLUMN,
        &DataType::Float32,
        false,
    )?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_RANKING_REASON_COLUMN,
        true,
    )?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_RETRIEVAL_MODE_COLUMN,
        true,
    )?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_EPISODIC_RECALL_SCHEMA_VERSION_COLUMN,
        false,
    )?;
    Ok(())
}

/// Validate one staged episodic recall response batch.
///
/// # Errors
///
/// Returns an error when the response batch violates the staged semantics.
pub fn validate_memory_julia_episodic_recall_response_batch(
    batch: &RecordBatch,
) -> Result<(), String> {
    validate_memory_julia_episodic_recall_response_schema(batch.schema().as_ref())?;

    require_non_blank_utf8_column(batch, MEMORY_JULIA_EPISODIC_RECALL_QUERY_ID_COLUMN)?;
    require_non_blank_utf8_column(batch, MEMORY_JULIA_EPISODIC_RECALL_CANDIDATE_ID_COLUMN)?;
    require_finite_float32_column(batch, MEMORY_JULIA_EPISODIC_RECALL_SEMANTIC_SCORE_COLUMN)?;
    require_finite_float32_column(batch, MEMORY_JULIA_EPISODIC_RECALL_UTILITY_SCORE_COLUMN)?;
    require_finite_float32_column(batch, MEMORY_JULIA_EPISODIC_RECALL_FINAL_SCORE_COLUMN)?;
    require_non_blank_optional_utf8_column(
        batch,
        MEMORY_JULIA_EPISODIC_RECALL_RANKING_REASON_COLUMN,
    )?;
    require_non_blank_optional_utf8_column(
        batch,
        MEMORY_JULIA_EPISODIC_RECALL_RETRIEVAL_MODE_COLUMN,
    )?;
    require_non_blank_utf8_column(batch, MEMORY_JULIA_EPISODIC_RECALL_SCHEMA_VERSION_COLUMN)?;

    let confidence = float32_column(batch, MEMORY_JULIA_EPISODIC_RECALL_CONFIDENCE_COLUMN)
        .map_err(|error| error.to_string())?;
    for row in 0..batch.num_rows() {
        let value = confidence.value(row);
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(format!(
                "episodic recall response row {row} has confidence outside [0, 1]"
            ));
        }
    }

    Ok(())
}

/// Validate many staged episodic recall response batches.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any response batch violates the staged
/// contract.
pub fn validate_memory_julia_episodic_recall_response_batches(
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    for batch in batches {
        validate_memory_julia_episodic_recall_response_batch(batch)
            .map_err(|error| episodic_recall_contract_error(&error))?;
    }
    Ok(())
}

/// Decode many staged episodic recall response batches into typed score rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any batch violates the staged
/// response contract.
pub fn decode_memory_julia_episodic_recall_score_rows(
    batches: &[RecordBatch],
) -> Result<Vec<MemoryJuliaEpisodicRecallScoreRow>, RepoIntelligenceError> {
    validate_memory_julia_episodic_recall_response_batches(batches)?;

    let mut rows = Vec::new();
    for batch in batches {
        let query_id = utf8_column(batch, MEMORY_JULIA_EPISODIC_RECALL_QUERY_ID_COLUMN)?;
        let candidate_id = utf8_column(batch, MEMORY_JULIA_EPISODIC_RECALL_CANDIDATE_ID_COLUMN)?;
        let semantic_score =
            float32_column(batch, MEMORY_JULIA_EPISODIC_RECALL_SEMANTIC_SCORE_COLUMN)?;
        let utility_score =
            float32_column(batch, MEMORY_JULIA_EPISODIC_RECALL_UTILITY_SCORE_COLUMN)?;
        let final_score = float32_column(batch, MEMORY_JULIA_EPISODIC_RECALL_FINAL_SCORE_COLUMN)?;
        let confidence = float32_column(batch, MEMORY_JULIA_EPISODIC_RECALL_CONFIDENCE_COLUMN)?;
        let ranking_reason =
            nullable_utf8_column(batch, MEMORY_JULIA_EPISODIC_RECALL_RANKING_REASON_COLUMN)?;
        let retrieval_mode =
            nullable_utf8_column(batch, MEMORY_JULIA_EPISODIC_RECALL_RETRIEVAL_MODE_COLUMN)?;
        let schema_version =
            utf8_column(batch, MEMORY_JULIA_EPISODIC_RECALL_SCHEMA_VERSION_COLUMN)?;

        for row in 0..batch.num_rows() {
            rows.push(MemoryJuliaEpisodicRecallScoreRow {
                query_id: query_id.value(row).into(),
                candidate_id: candidate_id.value(row).into(),
                semantic_score: semantic_score.value(row),
                utility_score: utility_score.value(row),
                final_score: final_score.value(row),
                confidence: confidence.value(row),
                ranking_reason: (!ranking_reason.is_null(row))
                    .then(|| ranking_reason.value(row).to_string()),
                retrieval_mode: (!retrieval_mode.is_null(row))
                    .then(|| retrieval_mode.value(row).into()),
                schema_version: schema_version.value(row).to_string(),
            });
        }
    }

    Ok(rows)
}

fn build_float32_list_array<'a>(values: impl Iterator<Item = &'a [f32]>) -> ListArray {
    let mut builder = ListBuilder::new(Float32Builder::new());
    for slice in values {
        for value in slice {
            builder.values().append_value(*value);
        }
        builder.append(true);
    }
    builder.finish()
}

fn validate_utf8_field(schema: &Schema, name: &str, nullable: bool) -> Result<(), String> {
    validate_primitive_field(schema, name, &DataType::Utf8, nullable)
}

fn validate_float32_list_field(schema: &Schema, name: &str, nullable: bool) -> Result<(), String> {
    let field = schema
        .field_with_name(name)
        .map_err(|_| format!("missing `{name}` field"))?;
    let expected = DataType::List(Arc::new(Field::new("item", DataType::Float32, true)));
    if field.data_type() != &expected {
        return Err(format!(
            "`{name}` must use {:?}, found {:?}",
            expected,
            field.data_type()
        ));
    }
    if field.is_nullable() != nullable {
        return Err(format!("`{name}` nullable mismatch"));
    }
    Ok(())
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
        .ok_or_else(|| episodic_recall_contract_error(&format!("missing `{name}` column")))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| episodic_recall_contract_error(&format!("`{name}` must be Utf8")))
}

fn nullable_utf8_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
) -> Result<&'a StringArray, RepoIntelligenceError> {
    utf8_column(batch, name)
}

fn float32_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
) -> Result<&'a Float32Array, RepoIntelligenceError> {
    batch
        .column_by_name(name)
        .ok_or_else(|| episodic_recall_contract_error(&format!("missing `{name}` column")))?
        .as_any()
        .downcast_ref::<Float32Array>()
        .ok_or_else(|| episodic_recall_contract_error(&format!("`{name}` must be Float32")))
}

fn int64_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
) -> Result<&'a Int64Array, RepoIntelligenceError> {
    batch
        .column_by_name(name)
        .ok_or_else(|| episodic_recall_contract_error(&format!("missing `{name}` column")))?
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| episodic_recall_contract_error(&format!("`{name}` must be Int64")))
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

fn require_finite_float32_column(batch: &RecordBatch, name: &str) -> Result<(), String> {
    let column = batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<Float32Array>()
        .ok_or_else(|| format!("`{name}` must be Float32"))?;

    for row in 0..batch.num_rows() {
        let value = column.value(row);
        if !value.is_finite() {
            return Err(format!("`{name}` contains a non-finite value at row {row}"));
        }
    }
    Ok(())
}

fn require_finite_non_negative_float32_column(
    batch: &RecordBatch,
    name: &str,
) -> Result<(), String> {
    let column = batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<Float32Array>()
        .ok_or_else(|| format!("`{name}` must be Float32"))?;

    for row in 0..batch.num_rows() {
        let value = column.value(row);
        if !value.is_finite() || value < 0.0 {
            return Err(format!(
                "`{name}` must contain finite non-negative values; found {value} at row {row}"
            ));
        }
    }
    Ok(())
}

fn require_non_empty_float32_list_column(batch: &RecordBatch, name: &str) -> Result<(), String> {
    let column = batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| format!("`{name}` must be List<Float32>"))?;

    for row in 0..batch.num_rows() {
        if column.is_null(row) {
            return Err(format!("`{name}` contains null at row {row}"));
        }
        let values = column.value(row);
        let values = values
            .as_any()
            .downcast_ref::<Float32Array>()
            .ok_or_else(|| format!("`{name}` inner values must be Float32"))?;
        if values.is_empty() {
            return Err(format!("`{name}` contains an empty embedding at row {row}"));
        }
        for index in 0..values.len() {
            let value = values.value(index);
            if !value.is_finite() {
                return Err(format!(
                    "`{name}` contains a non-finite embedding value at row {row}"
                ));
            }
        }
    }
    Ok(())
}

fn episodic_recall_contract_error(message: &str) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!("memory Julia episodic_recall contract violation: {message}"),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/memory/episodic_recall.rs"]
mod tests;
