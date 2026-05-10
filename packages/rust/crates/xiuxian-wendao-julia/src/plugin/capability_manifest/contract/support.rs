//! Shared support for the Julia capability-manifest Arrow contract.

use arrow::array::{Array, BooleanArray, StringArray, UInt64Array};
use arrow::record_batch::RecordBatch;
use serde_json::Value;
use xiuxian_wendao_core::{
    repo_intelligence::{RegisteredRepository, RepoIntelligenceError},
    transport::PluginTransportKind,
};

use crate::plugin::capability_manifest::{
    ARROW_FLIGHT_TRANSPORT_KIND, JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION,
};

pub(super) fn utf8_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
    direction: &str,
) -> Result<&'a StringArray, RepoIntelligenceError> {
    batch
        .column_by_name(name)
        .and_then(|array| array.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| {
            manifest_contract_error(direction, format!("missing required Utf8 column `{name}`"))
        })
}

pub(super) fn nullable_utf8_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
    direction: &str,
) -> Result<&'a StringArray, RepoIntelligenceError> {
    utf8_column(batch, name, direction)
}

pub(super) fn bool_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
    direction: &str,
) -> Result<&'a BooleanArray, RepoIntelligenceError> {
    batch
        .column_by_name(name)
        .and_then(|array| array.as_any().downcast_ref::<BooleanArray>())
        .ok_or_else(|| {
            manifest_contract_error(
                direction,
                format!("missing required Boolean column `{name}`"),
            )
        })
}

pub(super) fn nullable_u64_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
    direction: &str,
) -> Result<&'a UInt64Array, RepoIntelligenceError> {
    batch
        .column_by_name(name)
        .and_then(|array| array.as_any().downcast_ref::<UInt64Array>())
        .ok_or_else(|| {
            manifest_contract_error(
                direction,
                format!("missing required UInt64 column `{name}`"),
            )
        })
}

pub(super) fn string_value(array: &StringArray, row: usize) -> Option<&str> {
    (!array.is_null(row)).then(|| array.value(row))
}

pub(super) fn u64_value(array: &UInt64Array, row: usize) -> Option<u64> {
    (!array.is_null(row)).then(|| array.value(row))
}

pub(super) fn object_option<'a>(
    value: &'a Value,
    field: &str,
    repository: &RegisteredRepository,
) -> Result<&'a Value, RepoIntelligenceError> {
    if value.is_object() {
        return Ok(value);
    }

    Err(plugin_config_type_error(repository, field, "an object"))
}

pub(super) fn string_option(
    value: &Value,
    field: &str,
    repository: &RegisteredRepository,
) -> Result<Option<String>, RepoIntelligenceError> {
    let Some(raw) = value.get(field) else {
        return Ok(None);
    };
    let Some(string) = raw.as_str() else {
        return Err(plugin_config_type_error(repository, field, "a string"));
    };
    Ok(Some(string.to_string()))
}

pub(super) fn bool_option(
    value: &Value,
    field: &str,
    repository: &RegisteredRepository,
) -> Result<Option<bool>, RepoIntelligenceError> {
    let Some(raw) = value.get(field) else {
        return Ok(None);
    };
    let Some(boolean) = raw.as_bool() else {
        return Err(plugin_config_type_error(repository, field, "a boolean"));
    };
    Ok(Some(boolean))
}

pub(super) fn u64_option(
    value: &Value,
    field: &str,
    repository: &RegisteredRepository,
) -> Result<Option<u64>, RepoIntelligenceError> {
    let Some(raw) = value.get(field) else {
        return Ok(None);
    };
    let Some(number) = raw.as_u64() else {
        return Err(plugin_config_type_error(
            repository,
            field,
            "an unsigned integer",
        ));
    };
    Ok(Some(number))
}

pub(super) fn manifest_request_error(message: impl Into<String>) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "Julia capability-manifest request contract `{JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION}` violated: {}",
            message.into()
        ),
    }
}

pub(super) fn manifest_contract_error(
    direction: &str,
    message: impl Into<String>,
) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "Julia capability-manifest {direction} contract `{JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION}` violated: {}",
            message.into()
        ),
    }
}

pub(super) fn plugin_config_type_error(
    repository: &RegisteredRepository,
    field: &str,
    expected: &str,
) -> RepoIntelligenceError {
    RepoIntelligenceError::ConfigLoad {
        message: format!(
            "repo `{}` Julia plugin field `{field}` must be {expected}",
            repository.id
        ),
    }
}

pub(crate) fn parse_transport_kind(
    value: &str,
) -> Result<PluginTransportKind, RepoIntelligenceError> {
    match value {
        ARROW_FLIGHT_TRANSPORT_KIND => Ok(PluginTransportKind::ArrowFlight),
        other => Err(manifest_contract_error(
            "response",
            format!("unsupported `transport_kind` `{other}`"),
        )),
    }
}

pub(super) fn panic_payload_message(panic_payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = panic_payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = panic_payload.downcast_ref::<&'static str>() {
        return (*message).to_string();
    }
    "unknown panic payload".to_string()
}
