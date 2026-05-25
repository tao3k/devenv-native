//! Julia memory capability manifest transport contract and row processing.

use std::sync::Arc;

use arrow::array::{Array, BooleanArray, StringArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;
use xiuxian_wendao_runtime::{
    config::MemoryJuliaComputeRuntimeConfig,
    transport::{
        normalize_flight_route, validate_flight_schema_version, validate_flight_timeout_secs,
    },
};

use crate::wendao::{JuliaContractEnabled, JuliaContractId, JuliaContractSecondsU64};

use super::profile::{
    MEMORY_JULIA_COMPUTE_FAMILY_ID, MemoryJuliaComputeProfile, request_schema_id,
    response_schema_id,
};

/// Response column carrying the capability family id.
pub const MEMORY_JULIA_COMPUTE_MANIFEST_FAMILY_COLUMN: &str = "family";
/// Response column carrying the capability id.
pub const MEMORY_JULIA_COMPUTE_MANIFEST_CAPABILITY_ID_COLUMN: &str = "capability_id";
/// Response column carrying the profile id.
pub const MEMORY_JULIA_COMPUTE_MANIFEST_PROFILE_ID_COLUMN: &str = "profile_id";
/// Response column carrying the request schema id.
pub const MEMORY_JULIA_COMPUTE_MANIFEST_REQUEST_SCHEMA_ID_COLUMN: &str = "request_schema_id";
/// Response column carrying the response schema id.
pub const MEMORY_JULIA_COMPUTE_MANIFEST_RESPONSE_SCHEMA_ID_COLUMN: &str = "response_schema_id";
/// Response column carrying the route.
pub const MEMORY_JULIA_COMPUTE_MANIFEST_ROUTE_COLUMN: &str = "route";
/// Response column carrying the optional health route.
pub const MEMORY_JULIA_COMPUTE_MANIFEST_HEALTH_ROUTE_COLUMN: &str = "health_route";
/// Response column carrying the physical schema version.
pub const MEMORY_JULIA_COMPUTE_MANIFEST_SCHEMA_VERSION_COLUMN: &str = "schema_version";
/// Response column carrying the timeout in seconds.
pub const MEMORY_JULIA_COMPUTE_MANIFEST_TIMEOUT_SECS_COLUMN: &str = "timeout_secs";
/// Response column carrying the optional scenario pack.
pub const MEMORY_JULIA_COMPUTE_MANIFEST_SCENARIO_PACK_COLUMN: &str = "scenario_pack";
/// Response column carrying whether this row is enabled.
pub const MEMORY_JULIA_COMPUTE_MANIFEST_ENABLED_COLUMN: &str = "enabled";

/// Ordered response columns for the memory-family manifest projection.
pub const MEMORY_JULIA_COMPUTE_MANIFEST_RESPONSE_COLUMNS: [&str; 11] = [
    MEMORY_JULIA_COMPUTE_MANIFEST_FAMILY_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_CAPABILITY_ID_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_PROFILE_ID_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_REQUEST_SCHEMA_ID_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_RESPONSE_SCHEMA_ID_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_ROUTE_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_HEALTH_ROUTE_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_SCHEMA_VERSION_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_TIMEOUT_SECS_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_SCENARIO_PACK_COLUMN,
    MEMORY_JULIA_COMPUTE_MANIFEST_ENABLED_COLUMN,
];

/// One typed manifest row projected from runtime config plus profile metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryJuliaComputeManifestRow {
    /// Stable capability family id.
    pub family: String,
    /// Stable capability id for the staged profile.
    pub capability_id: JuliaContractId,
    /// Stable profile id.
    pub profile_id: JuliaContractId,
    /// Stable request schema id for semantic versioning.
    pub request_schema_id: JuliaContractId,
    /// Stable response schema id for semantic versioning.
    pub response_schema_id: JuliaContractId,
    /// Normalized Arrow Flight route.
    pub route: String,
    /// Optional health-check route.
    pub health_route: Option<String>,
    /// Physical transport schema version.
    pub schema_version: String,
    /// Optional timeout in seconds.
    pub timeout_secs: Option<JuliaContractSecondsU64>,
    /// Optional scenario pack hint.
    pub scenario_pack: Option<String>,
    /// Whether the runtime currently enables this profile.
    pub enabled: JuliaContractEnabled,
}

/// Project one manifest row per staged memory-family profile from runtime config.
#[must_use]
pub fn build_memory_julia_compute_manifest_rows(
    runtime: &MemoryJuliaComputeRuntimeConfig,
) -> Vec<MemoryJuliaComputeManifestRow> {
    MemoryJuliaComputeProfile::ALL
        .iter()
        .map(|profile| MemoryJuliaComputeManifestRow {
            family: MEMORY_JULIA_COMPUTE_FAMILY_ID.to_string(),
            capability_id: profile.capability_id().into(),
            profile_id: profile.profile_id().into(),
            request_schema_id: request_schema_id(*profile).into(),
            response_schema_id: response_schema_id(*profile).into(),
            route: route_for_profile(runtime, *profile).to_string(),
            health_route: runtime.health_route.clone(),
            schema_version: runtime.schema_version.clone(),
            timeout_secs: Some(runtime.timeout_secs.value().into()),
            scenario_pack: runtime.scenario_pack.clone(),
            enabled: runtime.enabled.into(),
        })
        .collect()
}

/// Build the memory-family manifest response schema.
#[must_use]
pub fn memory_julia_compute_manifest_response_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new(
            MEMORY_JULIA_COMPUTE_MANIFEST_FAMILY_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_COMPUTE_MANIFEST_CAPABILITY_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_COMPUTE_MANIFEST_PROFILE_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_COMPUTE_MANIFEST_REQUEST_SCHEMA_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_COMPUTE_MANIFEST_RESPONSE_SCHEMA_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_COMPUTE_MANIFEST_ROUTE_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_COMPUTE_MANIFEST_HEALTH_ROUTE_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            MEMORY_JULIA_COMPUTE_MANIFEST_SCHEMA_VERSION_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            MEMORY_JULIA_COMPUTE_MANIFEST_TIMEOUT_SECS_COLUMN,
            DataType::UInt64,
            true,
        ),
        Field::new(
            MEMORY_JULIA_COMPUTE_MANIFEST_SCENARIO_PACK_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            MEMORY_JULIA_COMPUTE_MANIFEST_ENABLED_COLUMN,
            DataType::Boolean,
            false,
        ),
    ]))
}

/// Build one manifest response batch from typed rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the batch cannot be materialized or
/// violates the manifest response contract.
pub fn build_memory_julia_compute_manifest_response_batch(
    rows: &[MemoryJuliaComputeManifestRow],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let batch = RecordBatch::try_new(
        memory_julia_compute_manifest_response_schema(),
        vec![
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.family.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.capability_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.profile_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.request_schema_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.response_schema_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.route.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.health_route.as_deref())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.schema_version.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter()
                    .map(|row| row.timeout_secs.map(JuliaContractSecondsU64::value))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.scenario_pack.as_deref())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(BooleanArray::from(
                rows.iter()
                    .map(|row| row.enabled.value())
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(|error| manifest_contract_error(&error.to_string()))?;

    validate_memory_julia_compute_manifest_response_batch(&batch)
        .map_err(|error| manifest_contract_error(&error))?;
    Ok(batch)
}

/// Validate the memory-family manifest response schema.
///
/// # Errors
///
/// Returns an error when the schema does not match the manifest response
/// contract.
pub fn validate_memory_julia_compute_manifest_response_schema(
    schema: &Schema,
) -> Result<(), String> {
    validate_utf8_field(schema, MEMORY_JULIA_COMPUTE_MANIFEST_FAMILY_COLUMN, false)?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_COMPUTE_MANIFEST_CAPABILITY_ID_COLUMN,
        false,
    )?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_COMPUTE_MANIFEST_PROFILE_ID_COLUMN,
        false,
    )?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_COMPUTE_MANIFEST_REQUEST_SCHEMA_ID_COLUMN,
        false,
    )?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_COMPUTE_MANIFEST_RESPONSE_SCHEMA_ID_COLUMN,
        false,
    )?;
    validate_utf8_field(schema, MEMORY_JULIA_COMPUTE_MANIFEST_ROUTE_COLUMN, false)?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_COMPUTE_MANIFEST_HEALTH_ROUTE_COLUMN,
        true,
    )?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_COMPUTE_MANIFEST_SCHEMA_VERSION_COLUMN,
        false,
    )?;
    validate_u64_field(
        schema,
        MEMORY_JULIA_COMPUTE_MANIFEST_TIMEOUT_SECS_COLUMN,
        true,
    )?;
    validate_utf8_field(
        schema,
        MEMORY_JULIA_COMPUTE_MANIFEST_SCENARIO_PACK_COLUMN,
        true,
    )?;
    validate_bool_field(schema, MEMORY_JULIA_COMPUTE_MANIFEST_ENABLED_COLUMN, false)?;
    Ok(())
}

/// Validate one manifest response batch.
///
/// # Errors
///
/// Returns an error when the batch violates the manifest response semantics.
pub fn validate_memory_julia_compute_manifest_response_batch(
    batch: &RecordBatch,
) -> Result<(), String> {
    validate_memory_julia_compute_manifest_response_schema(batch.schema().as_ref())?;

    if batch.num_rows() == 0 {
        return Err(
            "memory Julia compute manifest batch must contain at least one row".to_string(),
        );
    }

    let columns = ManifestResponseColumns::new(batch)?;
    for row in 0..batch.num_rows() {
        validate_manifest_response_row(&columns, row)?;
    }

    Ok(())
}

/// Validate many manifest response batches.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any batch violates the manifest
/// response contract.
pub fn validate_memory_julia_compute_manifest_response_batches(
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    for batch in batches {
        validate_memory_julia_compute_manifest_response_batch(batch)
            .map_err(|error| manifest_contract_error(&error))?;
    }
    Ok(())
}

/// Decode many manifest response batches into typed rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any batch violates the manifest
/// response contract.
pub fn decode_memory_julia_compute_manifest_rows(
    batches: &[RecordBatch],
) -> Result<Vec<MemoryJuliaComputeManifestRow>, RepoIntelligenceError> {
    validate_memory_julia_compute_manifest_response_batches(batches)?;

    let mut rows = Vec::new();
    for batch in batches {
        let family = utf8_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_FAMILY_COLUMN)?;
        let capability_id = utf8_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_CAPABILITY_ID_COLUMN)?;
        let profile_id = utf8_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_PROFILE_ID_COLUMN)?;
        let request_schema_id = utf8_column(
            batch,
            MEMORY_JULIA_COMPUTE_MANIFEST_REQUEST_SCHEMA_ID_COLUMN,
        )?;
        let response_schema_id = utf8_column(
            batch,
            MEMORY_JULIA_COMPUTE_MANIFEST_RESPONSE_SCHEMA_ID_COLUMN,
        )?;
        let route = utf8_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_ROUTE_COLUMN)?;
        let health_route = utf8_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_HEALTH_ROUTE_COLUMN)?;
        let schema_version =
            utf8_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_SCHEMA_VERSION_COLUMN)?;
        let timeout_secs = u64_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_TIMEOUT_SECS_COLUMN)?;
        let scenario_pack = utf8_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_SCENARIO_PACK_COLUMN)?;
        let enabled = bool_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_ENABLED_COLUMN)?;

        for row in 0..batch.num_rows() {
            rows.push(MemoryJuliaComputeManifestRow {
                family: family.value(row).to_string(),
                capability_id: capability_id.value(row).into(),
                profile_id: profile_id.value(row).into(),
                request_schema_id: request_schema_id.value(row).into(),
                response_schema_id: response_schema_id.value(row).into(),
                route: route.value(row).to_string(),
                health_route: optional_string_value(health_route, row)
                    .map_err(|error| manifest_contract_error(&error))?
                    .map(str::to_string),
                schema_version: schema_version.value(row).to_string(),
                timeout_secs: optional_u64_value(timeout_secs, row).map(Into::into),
                scenario_pack: optional_string_value(scenario_pack, row)
                    .map_err(|error| manifest_contract_error(&error))?
                    .map(str::to_string),
                enabled: enabled.value(row).into(),
            });
        }
    }

    Ok(rows)
}

fn route_for_profile(
    runtime: &MemoryJuliaComputeRuntimeConfig,
    profile: MemoryJuliaComputeProfile,
) -> &str {
    match profile {
        MemoryJuliaComputeProfile::EpisodicRecall => runtime.routes.episodic_recall.as_str(),
        MemoryJuliaComputeProfile::MemoryGateScore => runtime.routes.memory_gate_score.as_str(),
        MemoryJuliaComputeProfile::MemoryPlanTuning => runtime.routes.memory_plan_tuning.as_str(),
        MemoryJuliaComputeProfile::MemoryCalibration => runtime.routes.memory_calibration.as_str(),
    }
}

struct ManifestResponseColumns<'a> {
    family: &'a StringArray,
    capability_id: &'a StringArray,
    profile_id: &'a StringArray,
    request_schema_id: &'a StringArray,
    response_schema_id: &'a StringArray,
    route: &'a StringArray,
    health_route: &'a StringArray,
    schema_version: &'a StringArray,
    timeout_secs: &'a UInt64Array,
    scenario_pack: &'a StringArray,
    enabled: &'a BooleanArray,
}

impl<'a> ManifestResponseColumns<'a> {
    fn new(batch: &'a RecordBatch) -> Result<Self, String> {
        Ok(Self {
            family: utf8_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_FAMILY_COLUMN)
                .map_err(|error| error.to_string())?,
            capability_id: utf8_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_CAPABILITY_ID_COLUMN)
                .map_err(|error| error.to_string())?,
            profile_id: utf8_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_PROFILE_ID_COLUMN)
                .map_err(|error| error.to_string())?,
            request_schema_id: utf8_column(
                batch,
                MEMORY_JULIA_COMPUTE_MANIFEST_REQUEST_SCHEMA_ID_COLUMN,
            )
            .map_err(|error| error.to_string())?,
            response_schema_id: utf8_column(
                batch,
                MEMORY_JULIA_COMPUTE_MANIFEST_RESPONSE_SCHEMA_ID_COLUMN,
            )
            .map_err(|error| error.to_string())?,
            route: utf8_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_ROUTE_COLUMN)
                .map_err(|error| error.to_string())?,
            health_route: utf8_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_HEALTH_ROUTE_COLUMN)
                .map_err(|error| error.to_string())?,
            schema_version: utf8_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_SCHEMA_VERSION_COLUMN)
                .map_err(|error| error.to_string())?,
            timeout_secs: u64_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_TIMEOUT_SECS_COLUMN)
                .map_err(|error| error.to_string())?,
            scenario_pack: utf8_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_SCENARIO_PACK_COLUMN)
                .map_err(|error| error.to_string())?,
            enabled: bool_column(batch, MEMORY_JULIA_COMPUTE_MANIFEST_ENABLED_COLUMN)
                .map_err(|error| error.to_string())?,
        })
    }
}

fn validate_manifest_response_row(
    columns: &ManifestResponseColumns<'_>,
    row: usize,
) -> Result<(), String> {
    let family_value = require_non_blank_string_value(
        columns.family,
        row,
        MEMORY_JULIA_COMPUTE_MANIFEST_FAMILY_COLUMN,
    )?;
    if family_value != MEMORY_JULIA_COMPUTE_FAMILY_ID {
        return Err(format!(
            "`{MEMORY_JULIA_COMPUTE_MANIFEST_FAMILY_COLUMN}` must equal `{MEMORY_JULIA_COMPUTE_FAMILY_ID}` at row {row}"
        ));
    }

    let capability_id_value = require_non_blank_string_value(
        columns.capability_id,
        row,
        MEMORY_JULIA_COMPUTE_MANIFEST_CAPABILITY_ID_COLUMN,
    )?;
    let profile_id_value = require_non_blank_string_value(
        columns.profile_id,
        row,
        MEMORY_JULIA_COMPUTE_MANIFEST_PROFILE_ID_COLUMN,
    )?;
    let Some(profile) = MemoryJuliaComputeProfile::parse(profile_id_value) else {
        return Err(format!(
            "`{MEMORY_JULIA_COMPUTE_MANIFEST_PROFILE_ID_COLUMN}` contains unknown profile `{profile_id_value}` at row {row}"
        ));
    };
    validate_contract_match(
        capability_id_value,
        profile.capability_id(),
        MEMORY_JULIA_COMPUTE_MANIFEST_CAPABILITY_ID_COLUMN,
        row,
    )?;

    let request_schema_id_value = require_non_blank_string_value(
        columns.request_schema_id,
        row,
        MEMORY_JULIA_COMPUTE_MANIFEST_REQUEST_SCHEMA_ID_COLUMN,
    )?;
    validate_contract_match(
        request_schema_id_value,
        request_schema_id(profile),
        MEMORY_JULIA_COMPUTE_MANIFEST_REQUEST_SCHEMA_ID_COLUMN,
        row,
    )?;

    let response_schema_id_value = require_non_blank_string_value(
        columns.response_schema_id,
        row,
        MEMORY_JULIA_COMPUTE_MANIFEST_RESPONSE_SCHEMA_ID_COLUMN,
    )?;
    validate_contract_match(
        response_schema_id_value,
        response_schema_id(profile),
        MEMORY_JULIA_COMPUTE_MANIFEST_RESPONSE_SCHEMA_ID_COLUMN,
        row,
    )?;

    let route_value = require_non_blank_string_value(
        columns.route,
        row,
        MEMORY_JULIA_COMPUTE_MANIFEST_ROUTE_COLUMN,
    )?;
    normalize_flight_route(route_value)
        .map_err(|error| format!("`route` must be a normalized Flight route: {error}"))?;

    if let Some(health_route_value) = optional_string_value(columns.health_route, row)? {
        normalize_flight_route(health_route_value).map_err(|error| {
            format!("`health_route` must be a normalized Flight route: {error}")
        })?;
    }

    let schema_version_value = require_non_blank_string_value(
        columns.schema_version,
        row,
        MEMORY_JULIA_COMPUTE_MANIFEST_SCHEMA_VERSION_COLUMN,
    )?;
    validate_flight_schema_version(schema_version_value)
        .map_err(|error| format!("`schema_version` must be valid: {error}"))?;

    if let Some(timeout_secs_value) = optional_u64_value(columns.timeout_secs, row) {
        validate_flight_timeout_secs(timeout_secs_value)
            .map_err(|error| format!("`timeout_secs` must be valid: {error}"))?;
    }

    let _scenario_pack = optional_string_value(columns.scenario_pack, row)?;
    if columns.enabled.is_null(row) {
        return Err("`enabled` must be non-null".to_string());
    }

    Ok(())
}

fn validate_contract_match(
    actual: &str,
    expected: &str,
    column: &str,
    row: usize,
) -> Result<(), String> {
    if actual != expected {
        return Err(format!(
            "`{column}` must match staged profile contract at row {row}"
        ));
    }
    Ok(())
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

fn validate_u64_field(schema: &Schema, name: &str, nullable: bool) -> Result<(), String> {
    let field = schema
        .field_with_name(name)
        .map_err(|_| format!("missing `{name}` field"))?;
    if field.data_type() != &DataType::UInt64 {
        return Err(format!(
            "`{name}` must use {:?}, found {:?}",
            DataType::UInt64,
            field.data_type()
        ));
    }
    if field.is_nullable() != nullable {
        return Err(format!("`{name}` nullable mismatch"));
    }
    Ok(())
}

fn validate_bool_field(schema: &Schema, name: &str, nullable: bool) -> Result<(), String> {
    let field = schema
        .field_with_name(name)
        .map_err(|_| format!("missing `{name}` field"))?;
    if field.data_type() != &DataType::Boolean {
        return Err(format!(
            "`{name}` must use {:?}, found {:?}",
            DataType::Boolean,
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
        .ok_or_else(|| manifest_contract_error(&format!("missing `{name}` column")))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| manifest_contract_error(&format!("`{name}` must be Utf8")))
}

fn u64_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
) -> Result<&'a UInt64Array, RepoIntelligenceError> {
    batch
        .column_by_name(name)
        .ok_or_else(|| manifest_contract_error(&format!("missing `{name}` column")))?
        .as_any()
        .downcast_ref::<UInt64Array>()
        .ok_or_else(|| manifest_contract_error(&format!("`{name}` must be UInt64")))
}

fn bool_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
) -> Result<&'a BooleanArray, RepoIntelligenceError> {
    batch
        .column_by_name(name)
        .ok_or_else(|| manifest_contract_error(&format!("missing `{name}` column")))?
        .as_any()
        .downcast_ref::<BooleanArray>()
        .ok_or_else(|| manifest_contract_error(&format!("`{name}` must be Boolean")))
}

fn require_non_blank_string_value<'a>(
    array: &'a StringArray,
    row: usize,
    column: &str,
) -> Result<&'a str, String> {
    let Some(value) = optional_string_value(array, row)? else {
        return Err(format!("`{column}` contains a blank value at row {row}"));
    };
    Ok(value)
}

fn optional_string_value(array: &StringArray, row: usize) -> Result<Option<&str>, String> {
    if array.is_null(row) {
        return Ok(None);
    }

    let value = array.value(row).trim();
    if value.is_empty() {
        return Err(format!("Utf8 column contains a blank value at row {row}"));
    }
    Ok(Some(value))
}

fn optional_u64_value(array: &UInt64Array, row: usize) -> Option<u64> {
    (!array.is_null(row)).then(|| array.value(row))
}

fn manifest_contract_error(message: &str) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!("memory Julia capability-manifest contract violation: {message}"),
    }
}
