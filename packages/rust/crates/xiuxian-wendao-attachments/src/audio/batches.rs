//! Audio shard Arrow batch builders and decoders.

use std::sync::Arc;

use arrow::array::{Array, ArrayRef, Float64Array, Int32Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;

use super::types::{
    AUDIO_SHARD_INPUT_SCHEMA_VERSION, AUDIO_SHARD_RESULT_SCHEMA_VERSION, AudioShardInput,
    AudioShardMaterializedItem, AudioShardResult, AudioShardResultStatus, AudioShardWorkerProfile,
};

/// Build audio worker input rows from Rust-materialized audio shards.
#[must_use]
pub fn build_audio_shard_inputs(
    shards: &[AudioShardMaterializedItem],
    profile: &AudioShardWorkerProfile,
) -> Vec<AudioShardInput> {
    shards
        .iter()
        .map(|shard| {
            let manifest = &shard.manifest;
            AudioShardInput {
                contract_version: AUDIO_SHARD_INPUT_SCHEMA_VERSION.to_owned(),
                source_path: manifest.source_id.clone(),
                source_content_hash: manifest.source_sha256.clone(),
                shard_path: shard.output_path.to_string_lossy().into_owned(),
                shard_sha256: shard.shard_sha256.clone(),
                shard_profile: manifest
                    .cache_key
                    .split(':')
                    .next()
                    .unwrap_or("")
                    .to_owned(),
                task_profile: profile.task_profile.clone(),
                backend_profile: profile.backend_profile.clone(),
                preferred_languages: profile.preferred_languages.clone(),
                sample_rate_hz: manifest.sample_rate_hz,
                channels: manifest.channels,
                audio_format: manifest.audio_format.clone(),
                start_ms: manifest.start_ms,
                duration_ms: manifest.duration_ms,
                media_start_ms: manifest.media_start_ms,
                media_duration_ms: manifest.media_duration_ms,
                context_before_ms: manifest.context_before_ms,
                context_after_ms: manifest.context_after_ms,
                shard_element_id: manifest.shard_id.clone(),
                reading_order_key: manifest.reading_order_key.clone(),
            }
        })
        .collect()
}

/// # Errors
///
/// Returns an error if Arrow cannot build the audio worker input batch.
pub fn build_audio_shard_input_batch(inputs: &[AudioShardInput]) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        audio_shard_input_schema(),
        vec![
            input_string_column(inputs, |input| input.contract_version.clone()),
            input_string_column(inputs, |input| input.source_path.clone()),
            input_string_column(inputs, |input| input.source_content_hash.clone()),
            input_string_column(inputs, |input| input.shard_path.clone()),
            input_string_column(inputs, |input| input.shard_sha256.clone()),
            input_string_column(inputs, |input| input.shard_profile.clone()),
            input_string_column(inputs, |input| input.task_profile.clone()),
            input_string_column(inputs, |input| input.backend_profile.clone()),
            input_string_column(inputs, |input| input.preferred_languages.join(",")),
            input_i32_column(inputs, |input| i32::try_from(input.sample_rate_hz))?,
            input_i32_column(inputs, |input| Ok(i32::from(input.channels)))?,
            input_string_column(inputs, |input| input.audio_format.clone()),
            input_i64_column(inputs, |input| i64::try_from(input.start_ms))?,
            input_i64_column(inputs, |input| i64::try_from(input.duration_ms))?,
            input_i64_column(inputs, |input| i64::try_from(input.media_start_ms))?,
            input_i64_column(inputs, |input| i64::try_from(input.media_duration_ms))?,
            input_i64_column(inputs, |input| i64::try_from(input.context_before_ms))?,
            input_i64_column(inputs, |input| i64::try_from(input.context_after_ms))?,
            input_string_column(inputs, |input| input.shard_element_id.clone()),
            input_string_column(inputs, |input| input.reading_order_key.clone()),
        ],
    )
    .map_err(|error| format!("build audio shard input Arrow batch: {error}"))
}

/// # Errors
///
/// Returns an error if Arrow cannot build the audio worker result batch.
pub fn build_audio_shard_result_batch(results: &[AudioShardResult]) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        audio_shard_result_schema(),
        vec![
            result_string_column(results, |result| result.contract_version.clone()),
            result_string_column(results, |result| result.source_path.clone()),
            result_string_column(results, |result| result.source_content_hash.clone()),
            result_string_column(results, |result| result.shard_path.clone()),
            result_string_column(results, |result| result.shard_sha256.clone()),
            result_string_column(results, |result| result.shard_profile.clone()),
            result_string_column(results, |result| result.task_profile.clone()),
            result_string_column(results, |result| result.backend_profile.clone()),
            result_string_column(results, |result| result.status.as_str().to_owned()),
            Arc::new(StringArray::from(
                results
                    .iter()
                    .map(|result| result.text.as_deref())
                    .collect::<Vec<_>>(),
            )),
            result_string_column(results, |result| result.text_mime_type.clone()),
            Arc::new(Float64Array::from(
                results
                    .iter()
                    .map(|result| result.confidence)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                results
                    .iter()
                    .map(|result| result.error_message.as_deref())
                    .collect::<Vec<_>>(),
            )),
            result_string_column(results, |result| result.shard_element_id.clone()),
            result_string_column(results, |result| result.element_id.clone()),
        ],
    )
    .map_err(|error| format!("build audio shard result Arrow batch: {error}"))
}

/// Decode stable audio worker result rows from Arrow batches.
///
/// # Errors
///
/// Returns an error if any batch does not match the audio shard result schema or
/// contains unsupported contract values.
pub fn decode_audio_shard_result_batches(
    batches: &[RecordBatch],
) -> Result<Vec<AudioShardResult>, String> {
    let mut results = Vec::new();
    for batch in batches {
        results.extend(decode_audio_shard_result_batch(batch)?);
    }
    Ok(results)
}

/// Decode stable audio worker result rows from one Arrow batch.
///
/// # Errors
///
/// Returns an error if the batch does not match the audio shard result schema or
/// contains unsupported contract values.
pub fn decode_audio_shard_result_batch(
    batch: &RecordBatch,
) -> Result<Vec<AudioShardResult>, String> {
    validate_schema_compatible(
        batch.schema().as_ref(),
        audio_shard_result_schema().as_ref(),
        "audio shard result",
    )?;
    let columns = AudioShardResultColumns::from_batch(batch)?;
    (0..batch.num_rows())
        .map(|row| decode_audio_shard_result_row(&columns, row))
        .collect()
}

struct AudioShardResultColumns<'a> {
    contract_version: &'a StringArray,
    source_path: &'a StringArray,
    source_content_hash: &'a StringArray,
    shard_path: &'a StringArray,
    shard_sha256: &'a StringArray,
    shard_profile: &'a StringArray,
    task_profile: &'a StringArray,
    backend_profile: &'a StringArray,
    status: &'a StringArray,
    text: &'a StringArray,
    text_mime_type: &'a StringArray,
    confidence: &'a Float64Array,
    error_message: &'a StringArray,
    shard_element_id: &'a StringArray,
    element_id: &'a StringArray,
}

impl<'a> AudioShardResultColumns<'a> {
    fn from_batch(batch: &'a RecordBatch) -> Result<Self, String> {
        Ok(Self {
            contract_version: string_column(batch, "contractVersion")?,
            source_path: string_column(batch, "sourcePath")?,
            source_content_hash: string_column(batch, "sourceContentHash")?,
            shard_path: string_column(batch, "shardPath")?,
            shard_sha256: string_column(batch, "shardSha256")?,
            shard_profile: string_column(batch, "shardProfile")?,
            task_profile: string_column(batch, "taskProfile")?,
            backend_profile: string_column(batch, "backendProfile")?,
            status: string_column(batch, "status")?,
            text: string_column(batch, "text")?,
            text_mime_type: string_column(batch, "textMimeType")?,
            confidence: float64_column(batch, "confidence")?,
            error_message: string_column(batch, "errorMessage")?,
            shard_element_id: string_column(batch, "shardElementId")?,
            element_id: string_column(batch, "elementId")?,
        })
    }
}

fn decode_audio_shard_result_row(
    columns: &AudioShardResultColumns<'_>,
    row: usize,
) -> Result<AudioShardResult, String> {
    let version = required_string(columns.contract_version, row, "contractVersion")?;
    if version != AUDIO_SHARD_RESULT_SCHEMA_VERSION {
        return Err(format!(
            "unexpected audio shard result contract version `{version}`"
        ));
    }
    Ok(AudioShardResult {
        contract_version: version,
        source_path: required_string(columns.source_path, row, "sourcePath")?,
        source_content_hash: required_string(
            columns.source_content_hash,
            row,
            "sourceContentHash",
        )?,
        shard_path: required_string(columns.shard_path, row, "shardPath")?,
        shard_sha256: required_string(columns.shard_sha256, row, "shardSha256")?,
        shard_profile: required_string(columns.shard_profile, row, "shardProfile")?,
        task_profile: required_string(columns.task_profile, row, "taskProfile")?,
        backend_profile: required_string(columns.backend_profile, row, "backendProfile")?,
        status: AudioShardResultStatus::parse(
            required_string(columns.status, row, "status")?.as_str(),
        )?,
        text: optional_string(columns.text, row),
        text_mime_type: required_string(columns.text_mime_type, row, "textMimeType")?,
        confidence: optional_f64(columns.confidence, row),
        error_message: optional_string(columns.error_message, row),
        shard_element_id: required_string(columns.shard_element_id, row, "shardElementId")?,
        element_id: required_string(columns.element_id, row, "elementId")?,
    })
}

fn audio_shard_input_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        field_utf8("contractVersion", false),
        field_utf8("sourcePath", false),
        field_utf8("sourceContentHash", false),
        field_utf8("shardPath", false),
        field_utf8("shardSha256", false),
        field_utf8("shardProfile", false),
        field_utf8("taskProfile", false),
        field_utf8("backendProfile", false),
        field_utf8("preferredLanguages", false),
        Field::new("sampleRateHz", DataType::Int32, false),
        Field::new("channels", DataType::Int32, false),
        field_utf8("audioFormat", false),
        Field::new("startMs", DataType::Int64, false),
        Field::new("durationMs", DataType::Int64, false),
        Field::new("mediaStartMs", DataType::Int64, false),
        Field::new("mediaDurationMs", DataType::Int64, false),
        Field::new("contextBeforeMs", DataType::Int64, false),
        Field::new("contextAfterMs", DataType::Int64, false),
        field_utf8("shardElementId", false),
        field_utf8("readingOrderKey", false),
    ]))
}

fn audio_shard_result_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        field_utf8("contractVersion", false),
        field_utf8("sourcePath", false),
        field_utf8("sourceContentHash", false),
        field_utf8("shardPath", false),
        field_utf8("shardSha256", false),
        field_utf8("shardProfile", false),
        field_utf8("taskProfile", false),
        field_utf8("backendProfile", false),
        field_utf8("status", false),
        field_utf8("text", true),
        field_utf8("textMimeType", false),
        Field::new("confidence", DataType::Float64, true),
        field_utf8("errorMessage", true),
        field_utf8("shardElementId", false),
        field_utf8("elementId", false),
    ]))
}

fn field_utf8(name: &str, nullable: bool) -> Field {
    Field::new(name, DataType::Utf8, nullable)
}

fn input_string_column(
    inputs: &[AudioShardInput],
    value: impl Fn(&AudioShardInput) -> String,
) -> ArrayRef {
    Arc::new(StringArray::from(
        inputs.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn input_i32_column(
    inputs: &[AudioShardInput],
    value: impl Fn(&AudioShardInput) -> Result<i32, std::num::TryFromIntError>,
) -> Result<ArrayRef, String> {
    Ok(Arc::new(Int32Array::from(
        inputs
            .iter()
            .map(value)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("audio shard input value exceeds Int32: {error}"))?,
    )))
}

fn input_i64_column(
    inputs: &[AudioShardInput],
    value: impl Fn(&AudioShardInput) -> Result<i64, std::num::TryFromIntError>,
) -> Result<ArrayRef, String> {
    Ok(Arc::new(Int64Array::from(
        inputs
            .iter()
            .map(value)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("audio shard input value exceeds Int64: {error}"))?,
    )))
}

fn result_string_column(
    results: &[AudioShardResult],
    value: impl Fn(&AudioShardResult) -> String,
) -> ArrayRef {
    Arc::new(StringArray::from(
        results.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn validate_schema_compatible(
    actual: &Schema,
    expected: &Schema,
    label: &str,
) -> Result<(), String> {
    if actual.fields().len() != expected.fields().len() {
        return Err(format!(
            "{label} schema field count mismatch: expected {}, got {}",
            expected.fields().len(),
            actual.fields().len()
        ));
    }
    for (actual_field, expected_field) in actual.fields().iter().zip(expected.fields()) {
        if actual_field.name() != expected_field.name()
            || actual_field.data_type() != expected_field.data_type()
        {
            return Err(format!(
                "{label} schema mismatch at `{}`: got `{} {:?}`, expected `{} {:?}`",
                expected_field.name(),
                actual_field.name(),
                actual_field.data_type(),
                expected_field.name(),
                expected_field.data_type()
            ));
        }
    }
    Ok(())
}

fn string_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a StringArray, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("`{name}` column is not Utf8"))
}

fn float64_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Float64Array, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| format!("`{name}` column is not Float64"))
}

fn required_string(column: &StringArray, row: usize, name: &str) -> Result<String, String> {
    if column.is_null(row) {
        return Err(format!("`{name}` is null at row {row}"));
    }
    Ok(column.value(row).to_owned())
}

fn optional_string(column: &StringArray, row: usize) -> Option<String> {
    (!column.is_null(row)).then(|| column.value(row).to_owned())
}

fn optional_f64(column: &Float64Array, row: usize) -> Option<f64> {
    (!column.is_null(row)).then(|| column.value(row))
}
