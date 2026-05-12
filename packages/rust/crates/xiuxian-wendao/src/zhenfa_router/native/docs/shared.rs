//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use serde::Serialize;
use xiuxian_zhenfa::ZhenfaError;

pub(super) fn require_non_empty_argument(
    value: &str,
    field_name: &str,
) -> Result<String, ZhenfaError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ZhenfaError::invalid_arguments(format!(
            "`{field_name}` must be a non-empty string"
        )));
    }
    Ok(trimmed.to_string())
}

pub(super) fn optional_non_empty_argument(
    value: Option<String>,
    field_name: &str,
) -> Result<Option<String>, ZhenfaError> {
    value
        .map(|value| require_non_empty_argument(&value, field_name))
        .transpose()
}

pub(super) fn serialize_payload<T>(value: &T) -> Result<String, ZhenfaError>
where
    T: Serialize,
{
    serde_json::to_string_pretty(value).map_err(|error| {
        ZhenfaError::execution(format!("failed to serialize docs payload: {error}"))
    })
}
