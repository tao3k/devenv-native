//! Lightweight local SQL checkpoint persistence entrypoints.

use crate::checkpoint::{decode_checkpoint_json_impl, encode_checkpoint_json_impl};
use crate::checkpoint_api::BpmnCheckpointEnvelope;
use crate::error::{BpmnEngineError, Result};
use std::path::Path;
use xiuxian_db_store::rusqlite::{self, OptionalExtension};

const CHECKPOINT_SQL_TABLE: &str = "qianji_bpmn_checkpoints";

/// Loads a checkpoint envelope from the local SQL checkpoint store.
///
/// The current `sqlite` feature uses `SQLite` via `xiuxian-db-store` as a bounded
/// lightweight client-side persistence option.
///
/// # Errors
///
/// Returns [`BpmnEngineError::CheckpointStorage`] when the local database
/// cannot be opened or queried, or [`BpmnEngineError::CheckpointCodec`] when
/// the stored payload is not valid checkpoint JSON.
pub(in crate::checkpoint) fn load_checkpoint_sql_impl(
    instance_id: &str,
    database_path: &Path,
) -> Result<Option<BpmnCheckpointEnvelope>> {
    let connection = open_checkpoint_database(database_path, "load_checkpoint_sql_open")?;
    ensure_checkpoint_schema(&connection, "load_checkpoint_sql_schema")?;
    let payload: Option<String> = connection
        .query_row(
            &format!("SELECT payload_json FROM {CHECKPOINT_SQL_TABLE} WHERE instance_id = ?1"),
            [instance_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| BpmnEngineError::CheckpointStorage {
            operation: "load_checkpoint_sql_query",
            message: error.to_string(),
        })?;
    payload
        .as_deref()
        .map(decode_checkpoint_json_impl)
        .transpose()
}

/// Saves a checkpoint envelope to the local SQL checkpoint store.
///
/// The local SQL store keeps the same sequence-guard behavior as the Valkey
/// checkpoint path, but it intentionally does not expose distributed
/// lease-ownership helpers.
///
/// # Errors
///
/// Returns [`BpmnEngineError::StaleCheckpointWrite`] when the incoming
/// checkpoint sequence is not newer than the stored local checkpoint sequence,
/// [`BpmnEngineError::CheckpointStorage`] when the local database cannot be
/// opened or written, or [`BpmnEngineError::CheckpointCodec`] when the
/// checkpoint cannot be serialized.
pub(in crate::checkpoint) fn save_checkpoint_sql_impl(
    checkpoint: &BpmnCheckpointEnvelope,
    database_path: &Path,
) -> Result<()> {
    let mut connection = open_checkpoint_database(database_path, "save_checkpoint_sql_open")?;
    ensure_checkpoint_schema(&connection, "save_checkpoint_sql_schema")?;
    let payload = encode_checkpoint_json_impl(checkpoint)?;
    let sequence = sequence_to_i64(
        checkpoint.sequence,
        "save_checkpoint_sql_sequence_range",
        checkpoint.state.instance_id.as_ref(),
    )?;

    let transaction =
        connection
            .transaction()
            .map_err(|error| BpmnEngineError::CheckpointStorage {
                operation: "save_checkpoint_sql_begin",
                message: error.to_string(),
            })?;
    let instance_id = checkpoint.state.instance_id.as_ref();
    let stored_sequence: Option<i64> = transaction
        .query_row(
            &format!("SELECT sequence FROM {CHECKPOINT_SQL_TABLE} WHERE instance_id = ?1"),
            [instance_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| BpmnEngineError::CheckpointStorage {
            operation: "save_checkpoint_sql_read_sequence",
            message: error.to_string(),
        })?;

    if let Some(current_sequence) = stored_sequence
        && sequence <= current_sequence
    {
        return Err(BpmnEngineError::StaleCheckpointWrite {
            instance_id: instance_id.to_string(),
            attempted_sequence: checkpoint.sequence,
            stored_sequence: current_sequence.cast_unsigned(),
        });
    }

    transaction
        .execute(
            &format!(
                "INSERT INTO {CHECKPOINT_SQL_TABLE} (instance_id, sequence, payload_json)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(instance_id) DO UPDATE
                 SET sequence = excluded.sequence,
                     payload_json = excluded.payload_json"
            ),
            rusqlite::params![instance_id, sequence, payload],
        )
        .map_err(|error| BpmnEngineError::CheckpointStorage {
            operation: "save_checkpoint_sql_upsert",
            message: error.to_string(),
        })?;
    transaction
        .commit()
        .map_err(|error| BpmnEngineError::CheckpointStorage {
            operation: "save_checkpoint_sql_commit",
            message: error.to_string(),
        })?;
    Ok(())
}

/// Deletes a checkpoint envelope from the local SQL checkpoint store.
///
/// # Errors
///
/// Returns [`BpmnEngineError::CheckpointStorage`] when the local database
/// cannot be opened or written.
pub(in crate::checkpoint) fn delete_checkpoint_sql_impl(
    instance_id: &str,
    database_path: &Path,
) -> Result<()> {
    let connection = open_checkpoint_database(database_path, "delete_checkpoint_sql_open")?;
    ensure_checkpoint_schema(&connection, "delete_checkpoint_sql_schema")?;
    connection
        .execute(
            &format!("DELETE FROM {CHECKPOINT_SQL_TABLE} WHERE instance_id = ?1"),
            [instance_id],
        )
        .map_err(|error| BpmnEngineError::CheckpointStorage {
            operation: "delete_checkpoint_sql_delete",
            message: error.to_string(),
        })?;
    Ok(())
}

fn open_checkpoint_database(
    database_path: &Path,
    operation: &'static str,
) -> Result<rusqlite::Connection> {
    xiuxian_db_store::sql::open_sqlite_connection(database_path).map_err(|error| {
        BpmnEngineError::CheckpointStorage {
            operation,
            message: error.to_string(),
        }
    })
}

fn ensure_checkpoint_schema(
    connection: &rusqlite::Connection,
    operation: &'static str,
) -> Result<()> {
    connection
        .execute_batch(&format!(
            r"
CREATE TABLE IF NOT EXISTS {CHECKPOINT_SQL_TABLE} (
    instance_id TEXT PRIMARY KEY,
    sequence INTEGER NOT NULL,
    payload_json TEXT NOT NULL
);
"
        ))
        .map_err(|error| BpmnEngineError::CheckpointStorage {
            operation,
            message: error.to_string(),
        })
}

fn sequence_to_i64(sequence: u64, operation: &'static str, instance_id: &str) -> Result<i64> {
    i64::try_from(sequence).map_err(|error| BpmnEngineError::CheckpointStorage {
        operation,
        message: format!(
            "checkpoint sequence for instance '{instance_id}' does not fit in SQLite INTEGER: {error}"
        ),
    })
}
