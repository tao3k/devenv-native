//! Valkey persistence entrypoint shells.

use crate::checkpoint::{decode_checkpoint_json_impl, encode_checkpoint_json_impl};
use crate::checkpoint_api::BpmnCheckpointEnvelope;
use crate::error::{BpmnEngineError, Result};
use redis::AsyncCommands;

use super::{lease_key_impl, state_key_impl};

const CHECKPOINT_TTL_SECONDS: u64 = 604_800;
const SAVE_CHECKPOINT_CAS_SCRIPT: &str = r"
local current = redis.call('GET', KEYS[1])
local incoming_sequence = tonumber(ARGV[2])

if current then
  local ok, decoded = pcall(cjson.decode, current)
  if not ok then
    return redis.error_reply('checkpoint_state_decode_failed')
  end

  local current_sequence = tonumber(decoded['sequence'])
  if current_sequence and incoming_sequence <= current_sequence then
    return current_sequence
  end
end

redis.call('SETEX', KEYS[1], tonumber(ARGV[3]), ARGV[1])
return -1
";
const SAVE_CHECKPOINT_OWNED_CAS_SCRIPT: &str = r"
local lease_owner = redis.call('GET', KEYS[2])
if lease_owner ~= ARGV[4] then
  return -2
end

local current = redis.call('GET', KEYS[1])
local incoming_sequence = tonumber(ARGV[2])

if current then
  local ok, decoded = pcall(cjson.decode, current)
  if not ok then
    return redis.error_reply('checkpoint_state_decode_failed')
  end

  local current_sequence = tonumber(decoded['sequence'])
  if current_sequence and incoming_sequence <= current_sequence then
    return current_sequence
  end
end

redis.call('SETEX', KEYS[1], tonumber(ARGV[3]), ARGV[1])
return -1
";
const DELETE_CHECKPOINT_OWNED_SCRIPT: &str = r"
local lease_owner = redis.call('GET', KEYS[2])
if lease_owner ~= ARGV[1] then
  return 0
end

redis.call('DEL', KEYS[1])
return 1
";

/// Loads a checkpoint envelope from Valkey.
///
/// # Errors
///
/// Returns [`BpmnEngineError::CheckpointStorage`] when Valkey connectivity or
/// key lookup fails, or [`BpmnEngineError::CheckpointCodec`] when the stored
/// payload is not valid checkpoint JSON.
pub(in crate::checkpoint) async fn load_checkpoint_impl(
    instance_id: &str,
    valkey_url: &str,
) -> Result<Option<BpmnCheckpointEnvelope>> {
    let mut connection = connect_valkey_impl(valkey_url, "load_checkpoint_connect").await?;
    let payload: Option<String> =
        connection
            .get(state_key_impl(instance_id))
            .await
            .map_err(|error| BpmnEngineError::CheckpointStorage {
                operation: "load_checkpoint_get",
                message: error.to_string(),
            })?;
    payload
        .as_deref()
        .map(decode_checkpoint_json_impl)
        .transpose()
}

/// Saves a checkpoint envelope to Valkey.
///
/// # Errors
///
/// Returns [`BpmnEngineError::CheckpointStorage`] when Valkey connectivity or
/// key writes fail, or [`BpmnEngineError::CheckpointCodec`] when the checkpoint
/// cannot be serialized.
pub(in crate::checkpoint) async fn save_checkpoint_impl(
    checkpoint: &BpmnCheckpointEnvelope,
    valkey_url: &str,
) -> Result<()> {
    let mut connection = connect_valkey_impl(valkey_url, "save_checkpoint_connect").await?;
    let payload = encode_checkpoint_json_impl(checkpoint)?;
    let key = state_key_impl(checkpoint.state.instance_id.as_ref());
    let result: i64 = redis::cmd("EVAL")
        .arg(SAVE_CHECKPOINT_CAS_SCRIPT)
        .arg(1)
        .arg(&key)
        .arg(payload)
        .arg(checkpoint.sequence)
        .arg(CHECKPOINT_TTL_SECONDS)
        .query_async(&mut connection)
        .await
        .map_err(|error| BpmnEngineError::CheckpointStorage {
            operation: "save_checkpoint_eval",
            message: error.to_string(),
        })?;

    if result >= 0 {
        return Err(BpmnEngineError::StaleCheckpointWrite {
            instance_id: checkpoint.state.instance_id.to_string(),
            attempted_sequence: checkpoint.sequence,
            stored_sequence: result.cast_unsigned(),
        });
    }

    Ok(())
}

/// Deletes a checkpoint envelope from Valkey.
///
/// # Errors
///
/// Returns [`BpmnEngineError::CheckpointStorage`] when Valkey connectivity or
/// key deletion fails.
pub(in crate::checkpoint) async fn delete_checkpoint_impl(
    instance_id: &str,
    valkey_url: &str,
) -> Result<()> {
    let mut connection = connect_valkey_impl(valkey_url, "delete_checkpoint_connect").await?;
    let _: usize = connection
        .del(state_key_impl(instance_id))
        .await
        .map_err(|error| BpmnEngineError::CheckpointStorage {
            operation: "delete_checkpoint_del",
            message: error.to_string(),
        })?;
    Ok(())
}

/// Deletes a checkpoint envelope from Valkey when the caller owns the
/// checkpoint lease for that instance.
///
/// # Errors
///
/// Returns [`BpmnEngineError::CheckpointLeaseNotOwned`] when the caller does
/// not own the lease key, or [`BpmnEngineError::CheckpointStorage`] when
/// Valkey connectivity or key deletion fails.
pub(in crate::checkpoint) async fn delete_checkpoint_as_owner_impl(
    instance_id: &str,
    owner_token: &str,
    valkey_url: &str,
) -> Result<()> {
    let mut connection =
        connect_valkey_impl(valkey_url, "delete_checkpoint_as_owner_connect").await?;
    let deleted: i64 = redis::cmd("EVAL")
        .arg(DELETE_CHECKPOINT_OWNED_SCRIPT)
        .arg(2)
        .arg(state_key_impl(instance_id))
        .arg(lease_key_impl(instance_id))
        .arg(owner_token)
        .query_async(&mut connection)
        .await
        .map_err(|error| BpmnEngineError::CheckpointStorage {
            operation: "delete_checkpoint_as_owner_eval",
            message: error.to_string(),
        })?;

    match deleted {
        1 => Ok(()),
        0 => Err(BpmnEngineError::CheckpointLeaseNotOwned {
            instance_id: instance_id.to_string(),
        }),
        _ => Err(BpmnEngineError::CheckpointStorage {
            operation: "delete_checkpoint_as_owner_eval",
            message: "unexpected lease-guard delete result".to_string(),
        }),
    }
}

/// Saves a checkpoint envelope to Valkey when the caller owns the checkpoint
/// lease for that instance.
///
/// # Errors
///
/// Returns [`BpmnEngineError::CheckpointLeaseNotOwned`] when the caller does
/// not own the lease key, [`BpmnEngineError::StaleCheckpointWrite`] when the
/// incoming sequence is not newer than the stored checkpoint sequence,
/// [`BpmnEngineError::CheckpointStorage`] when Valkey connectivity or key
/// writes fail, or [`BpmnEngineError::CheckpointCodec`] when the checkpoint
/// cannot be serialized.
pub(in crate::checkpoint) async fn save_checkpoint_as_owner_impl(
    checkpoint: &BpmnCheckpointEnvelope,
    owner_token: &str,
    valkey_url: &str,
) -> Result<()> {
    let mut connection =
        connect_valkey_impl(valkey_url, "save_checkpoint_as_owner_connect").await?;
    let payload = encode_checkpoint_json_impl(checkpoint)?;
    let instance_id = checkpoint.state.instance_id.as_ref();
    let result: i64 = redis::cmd("EVAL")
        .arg(SAVE_CHECKPOINT_OWNED_CAS_SCRIPT)
        .arg(2)
        .arg(state_key_impl(instance_id))
        .arg(lease_key_impl(instance_id))
        .arg(payload)
        .arg(checkpoint.sequence)
        .arg(CHECKPOINT_TTL_SECONDS)
        .arg(owner_token)
        .query_async(&mut connection)
        .await
        .map_err(|error| BpmnEngineError::CheckpointStorage {
            operation: "save_checkpoint_as_owner_eval",
            message: error.to_string(),
        })?;

    match result {
        -1 => Ok(()),
        -2 => Err(BpmnEngineError::CheckpointLeaseNotOwned {
            instance_id: instance_id.to_string(),
        }),
        stored_sequence if stored_sequence >= 0 => Err(BpmnEngineError::StaleCheckpointWrite {
            instance_id: instance_id.to_string(),
            attempted_sequence: checkpoint.sequence,
            stored_sequence: stored_sequence.cast_unsigned(),
        }),
        _ => Err(BpmnEngineError::CheckpointStorage {
            operation: "save_checkpoint_as_owner_eval",
            message: "unexpected lease-guard save result".to_string(),
        }),
    }
}

pub(in crate::checkpoint) async fn connect_valkey_impl(
    valkey_url: &str,
    operation: &'static str,
) -> Result<redis::aio::MultiplexedConnection> {
    let client =
        redis::Client::open(valkey_url).map_err(|error| BpmnEngineError::CheckpointStorage {
            operation,
            message: error.to_string(),
        })?;
    client
        .get_multiplexed_async_connection()
        .await
        .map_err(|error| BpmnEngineError::CheckpointStorage {
            operation,
            message: error.to_string(),
        })
}
