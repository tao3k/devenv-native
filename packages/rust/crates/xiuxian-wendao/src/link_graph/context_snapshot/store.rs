//! `link_graph::context_snapshot::store` owns Wendao link graph context snapshot store behavior.

use crate::link_graph::context_snapshot::runtime::{
    normalize_snapshot_key_prefix, resolve_snapshot_runtime, snapshot_cache_key,
    snapshot_redis_client,
};
use crate::link_graph::context_snapshot::types::{
    QuantumContextSnapshot, quantum_context_snapshot_schema_fingerprint,
};
use crate::link_graph::models::QuantumContext;

fn decode_snapshot_payload(raw: &str, snapshot_id: &str) -> Option<QuantumContextSnapshot> {
    let parsed = serde_json::from_str::<QuantumContextSnapshot>(raw).ok()?;
    if parsed.schema_version
        != crate::link_graph::context_snapshot::LINK_GRAPH_QUANTUM_CONTEXT_SNAPSHOT_SCHEMA_VERSION
    {
        return None;
    }
    if parsed.schema_fingerprint != quantum_context_snapshot_schema_fingerprint() {
        return None;
    }
    if parsed.snapshot_id != snapshot_id {
        return None;
    }
    Some(parsed)
}

/// Read a quantum-context snapshot using runtime-configured Valkey settings.
///
/// # Errors
///
/// Returns an error when runtime configuration is invalid or Valkey operations fail.
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn valkey_quantum_context_snapshot_get(
    snapshot_id: &str,
) -> Result<Option<QuantumContextSnapshot>, String> {
    let (valkey_url, key_prefix) = resolve_snapshot_runtime()?;
    valkey_quantum_context_snapshot_get_with_valkey(snapshot_id, &valkey_url, Some(&key_prefix))
}

/// Read a quantum-context snapshot from a specific Valkey endpoint.
///
/// # Errors
///
/// Returns an error when inputs are invalid or Valkey operations fail.
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn valkey_quantum_context_snapshot_get_with_valkey(
    snapshot_id: &str,
    valkey_url: &str,
    key_prefix: Option<&str>,
) -> Result<Option<QuantumContextSnapshot>, String> {
    let trimmed = snapshot_id.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if valkey_url.trim().is_empty() {
        return Err("link_graph snapshot valkey_url must be non-empty".to_string());
    }
    let prefix = key_prefix.map(str::trim).map_or_else(
        || crate::link_graph::runtime_config::DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX.to_string(),
        normalize_snapshot_key_prefix,
    );
    let cache_key = snapshot_cache_key(trimmed, &prefix);

    let client = snapshot_redis_client(valkey_url)?;
    let mut conn = client
        .get_connection()
        .map_err(|err| format!("failed to connect valkey for link_graph snapshot store: {err}"))?;
    let raw = redis::cmd("GET")
        .arg(&cache_key)
        .query::<Option<String>>(&mut conn)
        .map_err(|err| format!("failed to GET link_graph snapshot: {err}"))?;
    let Some(payload_raw) = raw else {
        return Ok(None);
    };

    if let Some(snapshot) = decode_snapshot_payload(&payload_raw, trimmed) {
        return Ok(Some(snapshot));
    }

    let _ = redis::cmd("DEL").arg(&cache_key).query::<i64>(&mut conn);
    Ok(None)
}

/// Write a quantum-context snapshot using runtime-configured Valkey settings.
///
/// # Errors
///
/// Returns an error when runtime configuration is invalid or Valkey operations fail.
pub fn valkey_quantum_context_snapshot_save(
    snapshot: &QuantumContextSnapshot,
) -> Result<(), String> {
    let (valkey_url, key_prefix) = resolve_snapshot_runtime()?;
    valkey_quantum_context_snapshot_save_with_valkey(snapshot, &valkey_url, Some(&key_prefix))
}

/// Write a quantum-context snapshot to a specific Valkey endpoint.
///
/// # Errors
///
/// Returns an error when inputs are invalid, serialization fails, or Valkey operations fail.
pub fn valkey_quantum_context_snapshot_save_with_valkey(
    snapshot: &QuantumContextSnapshot,
    valkey_url: &str,
    key_prefix: Option<&str>,
) -> Result<(), String> {
    let snapshot_id = snapshot.snapshot_id.trim();
    if snapshot_id.is_empty() {
        return Err("snapshot_id must be non-empty".to_string());
    }
    if valkey_url.trim().is_empty() {
        return Err("link_graph snapshot valkey_url must be non-empty".to_string());
    }
    let prefix = key_prefix.map(str::trim).map_or_else(
        || crate::link_graph::runtime_config::DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX.to_string(),
        normalize_snapshot_key_prefix,
    );
    let cache_key = snapshot_cache_key(snapshot_id, &prefix);
    let encoded = serde_json::to_string(snapshot)
        .map_err(|err| format!("failed to serialize link_graph snapshot: {err}"))?;

    let client = snapshot_redis_client(valkey_url)?;
    let mut conn = client
        .get_connection()
        .map_err(|err| format!("failed to connect valkey for link_graph snapshot store: {err}"))?;
    redis::cmd("SET")
        .arg(&cache_key)
        .arg(&encoded)
        .query::<()>(&mut conn)
        .map_err(|err| format!("failed to SET link_graph snapshot: {err}"))?;
    Ok(())
}

/// Delete a quantum-context snapshot using runtime-configured Valkey settings.
///
/// # Errors
///
/// Returns an error when runtime configuration is invalid or Valkey operations fail.
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn valkey_quantum_context_snapshot_drop(snapshot_id: &str) -> Result<(), String> {
    let (valkey_url, key_prefix) = resolve_snapshot_runtime()?;
    let trimmed = snapshot_id.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    let cache_key = snapshot_cache_key(trimmed, &key_prefix);

    let client = snapshot_redis_client(valkey_url.as_str())?;
    let mut conn = client
        .get_connection()
        .map_err(|err| format!("failed to connect valkey for link_graph snapshot store: {err}"))?;
    redis::cmd("DEL")
        .arg(&cache_key)
        .query::<i64>(&mut conn)
        .map_err(|err| format!("failed to DEL link_graph snapshot: {err}"))?;
    Ok(())
}

/// Roll back to a stored quantum-context snapshot by id.
///
/// # Errors
///
/// Returns an error when runtime configuration is invalid or Valkey operations fail.
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn valkey_quantum_context_snapshot_rollback(
    snapshot_id: &str,
) -> Result<Option<Vec<QuantumContext>>, String> {
    let (valkey_url, key_prefix) = resolve_snapshot_runtime()?;
    valkey_quantum_context_snapshot_rollback_with_valkey(
        snapshot_id,
        &valkey_url,
        Some(&key_prefix),
    )
}

/// Roll back to a stored quantum-context snapshot using an explicit Valkey endpoint.
///
/// # Errors
///
/// Returns an error when inputs are invalid or Valkey operations fail.
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn valkey_quantum_context_snapshot_rollback_with_valkey(
    snapshot_id: &str,
    valkey_url: &str,
    key_prefix: Option<&str>,
) -> Result<Option<Vec<QuantumContext>>, String> {
    Ok(
        valkey_quantum_context_snapshot_get_with_valkey(snapshot_id, valkey_url, key_prefix)?
            .map(|snapshot| snapshot.contexts),
    )
}
