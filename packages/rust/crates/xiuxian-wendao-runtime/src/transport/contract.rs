use std::time::Duration;

/// Default base URL for a local Flight-backed Julia analyzer.
pub const DEFAULT_FLIGHT_BASE_URL: &str = "http://127.0.0.1:8815";
/// Default Wendao Flight schema contract version.
pub const DEFAULT_FLIGHT_SCHEMA_VERSION: &str = "v1";
/// Canonical Arrow schema metadata key for the Wendao schema version.
pub const FLIGHT_SCHEMA_VERSION_METADATA_KEY: &str = "wendao.schema_version";
/// Canonical Arrow schema metadata key for request/response trace identifiers.
pub const FLIGHT_TRACE_ID_METADATA_KEY: &str = "trace_id";
/// Default timeout for runtime-owned Flight roundtrips.
pub const DEFAULT_FLIGHT_TIMEOUT_SECS: u64 = 10;
/// Default maximum concurrent in-flight Flight roundtrips per transport client.
pub const DEFAULT_FLIGHT_MAX_IN_FLIGHT_REQUESTS: usize = 32;

/// Validate a non-empty Flight schema version string.
///
/// # Errors
///
/// Returns an error when the provided version is blank after trimming.
pub fn validate_flight_schema_version(schema_version: &str) -> Result<String, String> {
    let normalized = schema_version.trim();
    if normalized.is_empty() {
        return Err("Flight schema version must not be blank".to_string());
    }
    Ok(normalized.to_string())
}

/// Validate a non-zero timeout value for Flight roundtrips.
///
/// # Errors
///
/// Returns an error when the provided timeout is zero.
pub fn validate_flight_timeout_secs(timeout_secs: u64) -> Result<u64, String> {
    if timeout_secs == 0 {
        return Err("Flight timeout_secs must be greater than zero".to_string());
    }
    Ok(timeout_secs)
}

/// Validate a non-zero in-flight request budget for Flight roundtrips.
///
/// # Errors
///
/// Returns an error when the provided budget is zero or does not fit into the
/// current platform `usize`.
pub fn validate_flight_max_in_flight_requests(
    max_in_flight_requests: u64,
) -> Result<usize, String> {
    if max_in_flight_requests == 0 {
        return Err("Flight max_in_flight_requests must be greater than zero".to_string());
    }
    usize::try_from(max_in_flight_requests)
        .map_err(|_| "Flight max_in_flight_requests exceeds the current platform limit".to_string())
}

/// Resolve a runtime timeout from an optional `timeout_secs` override.
///
/// # Errors
///
/// Returns an error when the provided timeout override is zero.
pub fn resolve_flight_timeout(timeout_secs: Option<u64>) -> Result<Duration, String> {
    let timeout_secs = match timeout_secs {
        Some(timeout_secs) => validate_flight_timeout_secs(timeout_secs)?,
        None => DEFAULT_FLIGHT_TIMEOUT_SECS,
    };
    Ok(Duration::from_secs(timeout_secs))
}

/// Resolve one in-flight request budget from an optional override.
///
/// # Errors
///
/// Returns an error when the provided in-flight budget is zero or too large
/// for the current platform.
pub fn resolve_flight_max_in_flight_requests(
    max_in_flight_requests: Option<u64>,
) -> Result<usize, String> {
    match max_in_flight_requests {
        Some(max_in_flight_requests) => {
            validate_flight_max_in_flight_requests(max_in_flight_requests)
        }
        None => Ok(DEFAULT_FLIGHT_MAX_IN_FLIGHT_REQUESTS),
    }
}
