//! `valkey_common` owns Wendao valkey common behavior.

#[cfg(any(feature = "studio", feature = "zhenfa-router"))]
use std::time::Duration;

#[cfg(test)]
#[must_use]
fn first_non_empty_value<I>(values: I) -> Option<String>
where
    I: IntoIterator<Item = Option<String>>,
{
    values.into_iter().find_map(|candidate| {
        candidate
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

#[cfg(test)]
#[must_use]
fn open_optional_client(valkey_url: Option<String>) -> Option<redis::Client> {
    valkey_url.and_then(|value| open_client(value.as_str()).ok())
}

/// Open a Valkey client from one already-resolved endpoint URL.
///
/// # Errors
///
/// Returns the underlying `redis` client construction error when the URL is
/// invalid.
pub fn open_client(valkey_url: &str) -> Result<redis::Client, redis::RedisError> {
    redis::Client::open(valkey_url.trim())
}

#[cfg(any(feature = "studio", feature = "zhenfa-router"))]
/// Ping an already-open Valkey client with connection and IO timeouts.
///
/// # Errors
///
/// Returns a string error when the connection attempt or PING command fails.
pub fn ping_client(
    client: &redis::Client,
    connection_timeout: Duration,
    io_timeout: Duration,
) -> Result<String, String> {
    let connection = client
        .get_connection_with_timeout(connection_timeout)
        .map_err(|error| format!("connection failed: {error}"))?;
    let _ = connection.set_read_timeout(Some(io_timeout));
    let _ = connection.set_write_timeout(Some(io_timeout));
    let mut connection = connection;
    redis::cmd("PING")
        .query::<String>(&mut connection)
        .map_err(|error| format!("ping failed: {error}"))
}

#[cfg(any(feature = "studio", feature = "zhenfa-router"))]
/// Open and ping one Valkey URL with connection and IO timeouts.
///
/// # Errors
///
/// Returns a string error when the URL is invalid, the connection fails, or
/// the PING command fails.
pub fn ping_valkey_url(
    valkey_url: &str,
    connection_timeout: Duration,
    io_timeout: Duration,
) -> Result<String, String> {
    let client = open_client(valkey_url).map_err(|error| format!("invalid valkey url: {error}"))?;
    ping_client(&client, connection_timeout, io_timeout)
}

/// Normalize one optional key prefix with a required default.
#[must_use]
pub(crate) fn normalize_key_prefix(candidate: &str, default_prefix: &str) -> String {
    let normalized = candidate.trim();
    if normalized.is_empty() {
        default_prefix.to_string()
    } else {
        normalized.to_string()
    }
}

#[cfg(test)]
#[path = "../tests/unit/valkey_common.rs"]
mod tests;
