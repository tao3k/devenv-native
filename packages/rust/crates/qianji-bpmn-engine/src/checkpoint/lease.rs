//! Lease-key helpers for distributed checkpoint ownership.

use crate::error::{BpmnEngineError, Result};
use redis::AsyncCommands;

use super::keys::lease_key;
use super::valkey::connect_valkey;

const RENEW_LEASE_SCRIPT: &str = r#"
if redis.call("GET", KEYS[1]) == ARGV[1] then
  return redis.call("PEXPIRE", KEYS[1], ARGV[2])
else
  return 0
end
"#;

const RELEASE_LEASE_SCRIPT: &str = r#"
if redis.call("GET", KEYS[1]) == ARGV[1] then
  return redis.call("DEL", KEYS[1])
else
  return 0
end
"#;

/// Tries to acquire the BPMN checkpoint lease for one workflow instance.
///
/// # Errors
///
/// Returns [`BpmnEngineError::InvalidCheckpointLeaseTtl`] when `lease_ttl_ms`
/// is zero, or [`BpmnEngineError::CheckpointStorage`] when Valkey connectivity
/// or key writes fail.
pub async fn try_acquire_checkpoint_lease(
    instance_id: &str,
    owner_token: &str,
    lease_ttl_ms: u64,
    valkey_url: &str,
) -> Result<bool> {
    validate_lease_ttl_ms(lease_ttl_ms)?;
    let mut connection = connect_valkey(valkey_url, "try_acquire_checkpoint_lease_connect").await?;
    let acquired: Option<String> = connection
        .set_options(
            lease_key(instance_id),
            owner_token,
            redis::SetOptions::default()
                .conditional_set(redis::ExistenceCheck::NX)
                .with_expiration(redis::SetExpiry::PX(lease_ttl_ms)),
        )
        .await
        .map_err(|error| BpmnEngineError::CheckpointStorage {
            operation: "try_acquire_checkpoint_lease_set",
            message: error.to_string(),
        })?;
    Ok(acquired.is_some())
}

/// Renews the BPMN checkpoint lease when the caller still owns it.
///
/// # Errors
///
/// Returns [`BpmnEngineError::InvalidCheckpointLeaseTtl`] when `lease_ttl_ms`
/// is zero, or [`BpmnEngineError::CheckpointStorage`] when Valkey connectivity
/// or key writes fail.
pub async fn renew_checkpoint_lease(
    instance_id: &str,
    owner_token: &str,
    lease_ttl_ms: u64,
    valkey_url: &str,
) -> Result<bool> {
    validate_lease_ttl_ms(lease_ttl_ms)?;
    let mut connection = connect_valkey(valkey_url, "renew_checkpoint_lease_connect").await?;
    let renewed: i64 = redis::cmd("EVAL")
        .arg(RENEW_LEASE_SCRIPT)
        .arg(1)
        .arg(lease_key(instance_id))
        .arg(owner_token)
        .arg(lease_ttl_ms)
        .query_async(&mut connection)
        .await
        .map_err(|error| BpmnEngineError::CheckpointStorage {
            operation: "renew_checkpoint_lease_eval",
            message: error.to_string(),
        })?;
    Ok(renewed == 1)
}

/// Releases the BPMN checkpoint lease when the caller still owns it.
///
/// # Errors
///
/// Returns [`BpmnEngineError::CheckpointStorage`] when Valkey connectivity or
/// key writes fail.
pub async fn release_checkpoint_lease(
    instance_id: &str,
    owner_token: &str,
    valkey_url: &str,
) -> Result<bool> {
    let mut connection = connect_valkey(valkey_url, "release_checkpoint_lease_connect").await?;
    let released: i64 = redis::cmd("EVAL")
        .arg(RELEASE_LEASE_SCRIPT)
        .arg(1)
        .arg(lease_key(instance_id))
        .arg(owner_token)
        .query_async(&mut connection)
        .await
        .map_err(|error| BpmnEngineError::CheckpointStorage {
            operation: "release_checkpoint_lease_eval",
            message: error.to_string(),
        })?;
    Ok(released == 1)
}

fn validate_lease_ttl_ms(lease_ttl_ms: u64) -> Result<()> {
    if lease_ttl_ms == 0 {
        return Err(BpmnEngineError::InvalidCheckpointLeaseTtl {
            ttl_ms: lease_ttl_ms,
        });
    }
    Ok(())
}
