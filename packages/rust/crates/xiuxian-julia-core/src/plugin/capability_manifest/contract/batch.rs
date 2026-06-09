//! Arrow batch construction, validation, and decoding for Julia manifests.

use std::sync::Arc;

use arrow::array::{Array, ArrayRef, BooleanArray, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, build_arrow_schema,
    validate_record_batch_schema,
};
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;
use xiuxian_wendao_runtime::transport::{
    normalize_flight_route, validate_flight_schema_version, validate_flight_timeout_secs,
};

use crate::plugin::capability_manifest::{
    JULIA_PLUGIN_CAPABILITY_MANIFEST_BASE_URL_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_FILTER_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_ENABLED_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_HEALTH_ROUTE_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_INCLUDE_DISABLED_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_PLUGIN_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_REPOSITORY_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_PLUGIN_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_TRANSPORT_KIND_COLUMN,
    JuliaPluginCapabilityManifestRequestRow, JuliaPluginCapabilityManifestRow,
};

use super::support::{
    bool_column, manifest_contract_error, manifest_request_error, nullable_u64_column,
    nullable_utf8_column, parse_transport_kind, string_value, u64_value, utf8_column,
};

/// Build a Julia capability-manifest request batch from typed request rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the batch cannot be materialized or
/// violates the staged request contract.
pub fn build_julia_plugin_capability_manifest_request_batch(
    rows: &[JuliaPluginCapabilityManifestRequestRow],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let batch = RecordBatch::try_new(
        julia_plugin_capability_manifest_request_schema(),
        vec![
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.plugin_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.repository_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.capability_filter.as_deref())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(BooleanArray::from(
                rows.iter()
                    .map(|row| row.include_disabled.value())
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(|error| manifest_request_error(error.to_string()))?;
    validate_julia_plugin_capability_manifest_request_batches(std::slice::from_ref(&batch))?;
    Ok(batch)
}

/// Validate Julia capability-manifest request batches.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any request batch violates the
/// staged request contract.
pub fn validate_julia_plugin_capability_manifest_request_batches(
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    for batch in batches {
        validate_julia_plugin_capability_manifest_request_schema(batch)?;
        let columns = JuliaPluginCapabilityManifestRequestColumns::new(batch)?;
        validate_julia_plugin_capability_manifest_request_rows(&columns, batch.num_rows())?;
    }

    Ok(())
}

struct JuliaPluginCapabilityManifestRequestColumns<'a> {
    plugin_id: &'a StringArray,
    repository_id: &'a StringArray,
    include_disabled: &'a BooleanArray,
}

impl<'a> JuliaPluginCapabilityManifestRequestColumns<'a> {
    fn new(batch: &'a RecordBatch) -> Result<Self, RepoIntelligenceError> {
        let plugin_id = utf8_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_PLUGIN_ID_COLUMN,
            "request",
        )?;
        let repository_id = utf8_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_REPOSITORY_ID_COLUMN,
            "request",
        )?;
        let _capability_filter = nullable_utf8_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_FILTER_COLUMN,
            "request",
        )?;
        let include_disabled = bool_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_INCLUDE_DISABLED_COLUMN,
            "request",
        )?;
        Ok(Self {
            plugin_id,
            repository_id,
            include_disabled,
        })
    }
}

fn validate_julia_plugin_capability_manifest_request_rows(
    columns: &JuliaPluginCapabilityManifestRequestColumns<'_>,
    row_count: usize,
) -> Result<(), RepoIntelligenceError> {
    for row in 0..row_count {
        validate_julia_plugin_capability_manifest_request_row(columns, row)?;
    }
    Ok(())
}

fn validate_julia_plugin_capability_manifest_request_row(
    columns: &JuliaPluginCapabilityManifestRequestColumns<'_>,
    row: usize,
) -> Result<(), RepoIntelligenceError> {
    if columns.plugin_id.is_null(row) || columns.plugin_id.value(row).trim().is_empty() {
        return Err(manifest_contract_error(
            "request",
            "`plugin_id` must be non-null and non-blank",
        ));
    }
    if columns.repository_id.is_null(row) || columns.repository_id.value(row).trim().is_empty() {
        return Err(manifest_contract_error(
            "request",
            "`repository_id` must be non-null and non-blank",
        ));
    }
    if columns.include_disabled.is_null(row) {
        return Err(manifest_contract_error(
            "request",
            "`include_disabled` must be non-null",
        ));
    }
    Ok(())
}

/// Validate Julia capability-manifest response batches.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any response batch violates the
/// staged response contract.
pub fn validate_julia_plugin_capability_manifest_response_batches(
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    for batch in batches {
        let response_columns = JuliaPluginCapabilityManifestResponseColumns::new(batch)?;
        for row in 0..batch.num_rows() {
            validate_julia_plugin_capability_manifest_response_row(&response_columns, row)?;
        }
    }

    Ok(())
}

pub(super) fn normalize_julia_plugin_capability_manifest_response_batches(
    batches: &[RecordBatch],
) -> Result<Vec<RecordBatch>, RepoIntelligenceError> {
    batches
        .iter()
        .map(normalize_julia_plugin_capability_manifest_response_batch)
        .collect()
}

fn normalize_julia_plugin_capability_manifest_response_batch(
    batch: &RecordBatch,
) -> Result<RecordBatch, RepoIntelligenceError> {
    if batch
        .column_by_name(JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN)
        .is_some()
    {
        return Ok(batch.clone());
    }

    let mut fields = batch
        .schema()
        .fields()
        .iter()
        .map(|field| field.as_ref().clone())
        .collect::<Vec<_>>();
    let mut columns = batch.columns().to_vec();
    let insert_index = batch
        .schema()
        .index_of(JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_ID_COLUMN)
        .map_or(fields.len(), |index| index + 1);
    fields.insert(
        insert_index,
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN,
            DataType::Utf8,
            true,
        ),
    );
    columns.insert(
        insert_index,
        Arc::new(StringArray::from(vec![None::<&str>; batch.num_rows()])) as ArrayRef,
    );
    RecordBatch::try_new(Arc::new(Schema::new(fields)), columns).map_err(|error| {
        manifest_contract_error(
            "response",
            format!("failed to normalize legacy capability-manifest response batch: {error}"),
        )
    })
}

struct JuliaPluginCapabilityManifestResponseColumns<'a> {
    plugin_id: &'a StringArray,
    capability_id: &'a StringArray,
    capability_variant: &'a StringArray,
    transport_kind: &'a StringArray,
    base_url: &'a StringArray,
    route: &'a StringArray,
    health_route: &'a StringArray,
    schema_version: &'a StringArray,
    timeout_secs: &'a arrow::array::UInt64Array,
    enabled: &'a BooleanArray,
}

impl<'a> JuliaPluginCapabilityManifestResponseColumns<'a> {
    fn new(batch: &'a RecordBatch) -> Result<Self, RepoIntelligenceError> {
        Ok(Self {
            plugin_id: utf8_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_PLUGIN_ID_COLUMN,
                "response",
            )?,
            capability_id: utf8_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_ID_COLUMN,
                "response",
            )?,
            capability_variant: nullable_utf8_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN,
                "response",
            )?,
            transport_kind: utf8_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_TRANSPORT_KIND_COLUMN,
                "response",
            )?,
            base_url: utf8_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_BASE_URL_COLUMN,
                "response",
            )?,
            route: utf8_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE_COLUMN,
                "response",
            )?,
            health_route: nullable_utf8_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_HEALTH_ROUTE_COLUMN,
                "response",
            )?,
            schema_version: utf8_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION_COLUMN,
                "response",
            )?,
            timeout_secs: nullable_u64_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN,
                "response",
            )?,
            enabled: bool_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_ENABLED_COLUMN,
                "response",
            )?,
        })
    }
}

fn validate_julia_plugin_capability_manifest_response_row(
    columns: &JuliaPluginCapabilityManifestResponseColumns<'_>,
    row: usize,
) -> Result<(), RepoIntelligenceError> {
    let _capability_variant = string_value(columns.capability_variant, row);
    validate_non_blank_manifest_response_value(
        columns.plugin_id,
        row,
        "`plugin_id` must be non-null and non-blank",
    )?;
    validate_non_blank_manifest_response_value(
        columns.capability_id,
        row,
        "`capability_id` must be non-null and non-blank",
    )?;
    let transport_kind = validate_non_blank_manifest_response_value(
        columns.transport_kind,
        row,
        "`transport_kind` must be non-null and non-blank",
    )?;
    parse_transport_kind(transport_kind)?;
    validate_non_blank_manifest_response_value(
        columns.base_url,
        row,
        "`base_url` must be non-null and non-blank",
    )?;
    let route = validate_non_blank_manifest_response_value(
        columns.route,
        row,
        "`route` must be non-null and non-blank",
    )?;
    validate_manifest_response_route(route, "`route` must be a normalized Flight route")?;
    if let Some(health_route) = string_value(columns.health_route, row) {
        validate_manifest_response_route(
            health_route,
            "`health_route` must be a normalized Flight route",
        )?;
    }
    let schema_version = validate_non_blank_manifest_response_value(
        columns.schema_version,
        row,
        "`schema_version` must be non-null and non-blank",
    )?;
    validate_flight_schema_version(schema_version).map_err(|error| {
        manifest_contract_error(
            "response",
            format!("`schema_version` must be valid: {error}"),
        )
    })?;
    if let Some(timeout_secs) = u64_value(columns.timeout_secs, row) {
        validate_flight_timeout_secs(timeout_secs).map_err(|error| {
            manifest_contract_error("response", format!("`timeout_secs` must be valid: {error}"))
        })?;
    }
    if columns.enabled.is_null(row) {
        return Err(manifest_contract_error(
            "response",
            "`enabled` must be non-null",
        ));
    }
    Ok(())
}

fn validate_non_blank_manifest_response_value<'a>(
    array: &'a StringArray,
    row: usize,
    error_message: &str,
) -> Result<&'a str, RepoIntelligenceError> {
    let value = string_value(array, row)
        .ok_or_else(|| manifest_contract_error("response", error_message))?;
    if value.trim().is_empty() {
        return Err(manifest_contract_error("response", error_message));
    }
    Ok(value)
}

fn validate_manifest_response_route(
    route: &str,
    prefix: &str,
) -> Result<(), RepoIntelligenceError> {
    normalize_flight_route(route)
        .map_err(|error| manifest_contract_error("response", format!("{prefix}: {error}")))?;
    Ok(())
}

/// Decode response batches from the Julia capability-manifest route.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the staged response contract is
/// violated.
pub fn decode_julia_plugin_capability_manifest_rows(
    batches: &[RecordBatch],
) -> Result<Vec<JuliaPluginCapabilityManifestRow>, RepoIntelligenceError> {
    let batches = normalize_julia_plugin_capability_manifest_response_batches(batches)?;
    validate_julia_plugin_capability_manifest_response_batches(batches.as_slice())?;

    let mut rows = Vec::new();
    for batch in &batches {
        push_decoded_manifest_rows(batch, &mut rows)?;
    }

    Ok(rows)
}

fn push_decoded_manifest_rows(
    batch: &RecordBatch,
    rows: &mut Vec<JuliaPluginCapabilityManifestRow>,
) -> Result<(), RepoIntelligenceError> {
    let columns = JuliaPluginCapabilityManifestResponseColumns::new(batch)?;
    for row in 0..batch.num_rows() {
        rows.push(JuliaPluginCapabilityManifestRow {
            plugin_id: columns.plugin_id.value(row).into(),
            capability_id: columns.capability_id.value(row).into(),
            capability_variant: string_value(columns.capability_variant, row).map(Into::into),
            transport_kind: columns.transport_kind.value(row).into(),
            base_url: columns.base_url.value(row).into(),
            route: columns.route.value(row).into(),
            health_route: string_value(columns.health_route, row).map(Into::into),
            schema_version: columns.schema_version.value(row).into(),
            timeout_secs: u64_value(columns.timeout_secs, row).map(Into::into),
            enabled: columns.enabled.value(row).into(),
        });
    }
    Ok(())
}

fn julia_plugin_capability_manifest_request_schema() -> Arc<Schema> {
    Arc::new(build_arrow_schema(
        &julia_plugin_capability_manifest_request_contract(),
        std::collections::HashMap::new(),
    ))
}

fn validate_julia_plugin_capability_manifest_request_schema(
    batch: &RecordBatch,
) -> Result<(), RepoIntelligenceError> {
    validate_record_batch_schema(batch, &julia_plugin_capability_manifest_request_contract())
        .map_err(|error| {
            manifest_contract_error(
                "request",
                format!("capability-manifest request schema drift: {error}"),
            )
        })
}

fn julia_plugin_capability_manifest_request_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        "julia_plugin_capability_manifest_request",
        true,
        vec![
            ArrowSchemaColumn::new(
                JULIA_PLUGIN_CAPABILITY_MANIFEST_PLUGIN_ID_COLUMN,
                ArrowSchemaDataType::Utf8,
            ),
            ArrowSchemaColumn::new(
                JULIA_PLUGIN_CAPABILITY_MANIFEST_REPOSITORY_ID_COLUMN,
                ArrowSchemaDataType::Utf8,
            ),
            ArrowSchemaColumn::nullable(
                JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_FILTER_COLUMN,
                ArrowSchemaDataType::Utf8,
            ),
            ArrowSchemaColumn::new(
                JULIA_PLUGIN_CAPABILITY_MANIFEST_INCLUDE_DISABLED_COLUMN,
                ArrowSchemaDataType::Boolean,
            ),
        ],
    )
}
