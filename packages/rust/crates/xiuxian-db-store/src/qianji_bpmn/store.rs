//! DuckDB-backed Qianji BPMN workflow data-store implementation.

use crate::duckdb::DuckDbConnection;
use crate::duckdb_crate::OptionalExt;

use super::{
    QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY, QianjiBpmnDataRecord, QianjiBpmnDataStoreError,
    QianjiBpmnDuckDbDataStoreConfig,
};
use qianji_bpmn_engine::BpmnCheckpointEnvelope;

const WORKFLOW_DATA_TABLE: &str = "qianji_bpmn_workflow_data_records";
const UPSERT_WORKFLOW_DATA_RECORD_SQL: &str = "
INSERT INTO qianji_bpmn_workflow_data_records
    (instance_id, record_key, payload_json, updated_at_ms)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT(instance_id, record_key) DO UPDATE
SET payload_json = excluded.payload_json,
    updated_at_ms = excluded.updated_at_ms";
const LOAD_WORKFLOW_DATA_RECORD_SQL: &str = "
SELECT payload_json, updated_at_ms
FROM qianji_bpmn_workflow_data_records
WHERE instance_id = ?1 AND record_key = ?2";
const LOAD_WORKFLOW_STATE_SQL: &str = "
SELECT payload_json
FROM qianji_bpmn_workflow_data_records
WHERE instance_id = ?1 AND record_key = ?2";
const DELETE_WORKFLOW_DATA_RECORD_SQL: &str = "
DELETE FROM qianji_bpmn_workflow_data_records
WHERE instance_id = ?1 AND record_key = ?2";

/// DuckDB-backed BPMN workflow-local data store.
pub struct QianjiBpmnDuckDbDataStore {
    connection: DuckDbConnection,
}

impl QianjiBpmnDuckDbDataStore {
    /// Opens a workflow data store from one resolved `DuckDB` config.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when `DuckDB` cannot open or
    /// initialize the workflow data schema.
    pub fn open(config: QianjiBpmnDuckDbDataStoreConfig) -> Result<Self, QianjiBpmnDataStoreError> {
        let connection =
            DuckDbConnection::from_runtime(config.into_runtime()).map_err(|error| {
                QianjiBpmnDataStoreError::Storage {
                    operation: "open_duckdb_workflow_data_store",
                    message: error,
                }
            })?;
        let store = Self { connection };
        store.ensure_schema()?;
        store.ensure_workflow_state_log_schema()?;
        Ok(store)
    }

    /// Persists one workflow-local JSON record.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when the record is invalid, cannot
    /// be serialized, or `DuckDB` rejects the upsert.
    pub fn upsert_record(
        &self,
        record: &QianjiBpmnDataRecord,
    ) -> Result<(), QianjiBpmnDataStoreError> {
        validate_record(record)?;
        let payload_json = serde_json::to_string(&record.payload).map_err(|error| {
            QianjiBpmnDataStoreError::Codec {
                operation: "serialize_workflow_data_payload",
                message: error.to_string(),
            }
        })?;
        let updated_at_ms = timestamp_to_i64(&record.record_key, record.updated_at_ms)?;
        self.upsert_record_parts(
            &record.instance_id,
            &record.record_key,
            &payload_json,
            updated_at_ms,
        )
    }

    fn upsert_record_parts(
        &self,
        instance_id: &str,
        record_key: &str,
        payload_json: &str,
        updated_at_ms: i64,
    ) -> Result<(), QianjiBpmnDataStoreError> {
        let mut statement = self
            .connection()
            .prepare_cached(UPSERT_WORKFLOW_DATA_RECORD_SQL)
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "prepare_upsert_workflow_data_record",
                message: error.to_string(),
            })?;
        statement
            .execute(crate::duckdb_crate::params![
                instance_id,
                record_key,
                payload_json,
                updated_at_ms,
            ])
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "upsert_workflow_data_record",
                message: error.to_string(),
            })?;
        Ok(())
    }

    /// Loads one workflow-local JSON record by instance id and key.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when the key fields are invalid,
    /// `DuckDB` rejects the query, or the stored JSON payload cannot be decoded.
    pub fn load_record(
        &self,
        instance_id: &str,
        record_key: &str,
    ) -> Result<Option<QianjiBpmnDataRecord>, QianjiBpmnDataStoreError> {
        validate_field("instance_id", instance_id)?;
        validate_field("record_key", record_key)?;
        let mut statement = self
            .connection()
            .prepare_cached(LOAD_WORKFLOW_DATA_RECORD_SQL)
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "prepare_load_workflow_data_record",
                message: error.to_string(),
            })?;
        let row: Option<(String, i64)> = statement
            .query_row(
                crate::duckdb_crate::params![instance_id, record_key],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "load_workflow_data_record",
                message: error.to_string(),
            })?;
        row.map(|(payload_json, updated_at_ms)| {
            decode_record(instance_id, record_key, &payload_json, updated_at_ms)
        })
        .transpose()
    }

    /// Deletes one workflow-local JSON record by instance id and key.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when the key fields are invalid or
    /// `DuckDB` rejects the delete.
    pub fn delete_record(
        &self,
        instance_id: &str,
        record_key: &str,
    ) -> Result<bool, QianjiBpmnDataStoreError> {
        validate_field("instance_id", instance_id)?;
        validate_field("record_key", record_key)?;
        let mut statement = self
            .connection()
            .prepare_cached(DELETE_WORKFLOW_DATA_RECORD_SQL)
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "prepare_delete_workflow_data_record",
                message: error.to_string(),
            })?;
        let changed = statement
            .execute(crate::duckdb_crate::params![instance_id, record_key])
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "delete_workflow_data_record",
                message: error.to_string(),
            })?;
        Ok(changed > 0)
    }

    /// Persists the latest local no-server workflow-state snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when the snapshot cannot be
    /// serialized or the underlying `DuckDB` record upsert fails.
    pub fn upsert_workflow_state(
        &self,
        checkpoint: &BpmnCheckpointEnvelope,
    ) -> Result<(), QianjiBpmnDataStoreError> {
        let record = workflow_state_record_parts(checkpoint)?;
        self.upsert_record_parts(
            &record.instance_id,
            QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY,
            &record.payload_json,
            record.updated_at_ms,
        )
    }

    /// Persists multiple local no-server workflow-state snapshots in one transaction.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when the transaction cannot start,
    /// any snapshot cannot be serialized, or `DuckDB` rejects the batch commit.
    pub fn upsert_workflow_states<'a>(
        &self,
        checkpoints: impl IntoIterator<Item = &'a BpmnCheckpointEnvelope>,
    ) -> Result<usize, QianjiBpmnDataStoreError> {
        let records = checkpoints
            .into_iter()
            .map(workflow_state_record_parts)
            .collect::<Result<Vec<_>, QianjiBpmnDataStoreError>>()?;
        self.execute_in_transaction("upsert_workflow_state_snapshots_batch", || {
            let mut statement = self
                .connection()
                .prepare_cached(UPSERT_WORKFLOW_DATA_RECORD_SQL)
                .map_err(|error| QianjiBpmnDataStoreError::Storage {
                    operation: "prepare_upsert_workflow_state_snapshots_batch",
                    message: error.to_string(),
                })?;
            for record in &records {
                statement
                    .execute(crate::duckdb_crate::params![
                        record.instance_id.as_str(),
                        QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY,
                        record.payload_json.as_str(),
                        record.updated_at_ms,
                    ])
                    .map_err(|error| QianjiBpmnDataStoreError::Storage {
                        operation: "upsert_workflow_state_snapshots_batch",
                        message: error.to_string(),
                    })?;
            }
            Ok(records.len())
        })
    }

    /// Loads the latest local no-server workflow-state snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when the lookup fails or the stored
    /// JSON payload cannot be decoded into a checkpoint envelope.
    pub fn load_workflow_state(
        &self,
        instance_id: &str,
    ) -> Result<Option<BpmnCheckpointEnvelope>, QianjiBpmnDataStoreError> {
        validate_field("instance_id", instance_id)?;
        let mut statement = self
            .connection()
            .prepare_cached(LOAD_WORKFLOW_STATE_SQL)
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "prepare_load_workflow_state_snapshot",
                message: error.to_string(),
            })?;
        let payload_json: Option<String> = statement
            .query_row(
                crate::duckdb_crate::params![instance_id, QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "load_workflow_state_snapshot",
                message: error.to_string(),
            })?;
        payload_json
            .map(|payload_json| {
                serde_json::from_str(&payload_json).map_err(|error| {
                    QianjiBpmnDataStoreError::Codec {
                        operation: "decode_workflow_state_snapshot",
                        message: error.to_string(),
                    }
                })
            })
            .transpose()
    }

    /// Deletes the latest local no-server workflow-state snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when the underlying `DuckDB` delete
    /// fails.
    pub fn delete_workflow_state(
        &self,
        instance_id: &str,
    ) -> Result<bool, QianjiBpmnDataStoreError> {
        self.delete_record(instance_id, QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY)
    }

    /// Deletes multiple local no-server workflow-state snapshots in one transaction.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when the transaction cannot start,
    /// an instance id is invalid, or `DuckDB` rejects the batch commit.
    pub fn delete_workflow_states<'a>(
        &self,
        instance_ids: impl IntoIterator<Item = &'a str>,
    ) -> Result<usize, QianjiBpmnDataStoreError> {
        let instance_ids = instance_ids
            .into_iter()
            .map(|instance_id| {
                validate_field("instance_id", instance_id)?;
                Ok(instance_id.to_string())
            })
            .collect::<Result<Vec<_>, QianjiBpmnDataStoreError>>()?;
        self.execute_in_transaction("delete_workflow_state_snapshots_batch", || {
            let mut count = 0;
            let mut statement = self
                .connection()
                .prepare_cached(DELETE_WORKFLOW_DATA_RECORD_SQL)
                .map_err(|error| QianjiBpmnDataStoreError::Storage {
                    operation: "prepare_delete_workflow_state_snapshots_batch",
                    message: error.to_string(),
                })?;
            for instance_id in &instance_ids {
                let changed = statement
                    .execute(crate::duckdb_crate::params![
                        instance_id,
                        QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY,
                    ])
                    .map_err(|error| QianjiBpmnDataStoreError::Storage {
                        operation: "delete_workflow_state_snapshots_batch",
                        message: error.to_string(),
                    })?;
                if changed > 0 {
                    count += 1;
                }
            }
            Ok(count)
        })
    }

    pub(super) fn connection(&self) -> &crate::duckdb_crate::Connection {
        self.connection.connection()
    }

    pub(super) fn execute_in_transaction<T>(
        &self,
        operation: &'static str,
        action: impl FnOnce() -> Result<T, QianjiBpmnDataStoreError>,
    ) -> Result<T, QianjiBpmnDataStoreError> {
        self.execute_batch(operation, "BEGIN TRANSACTION")?;
        match action() {
            Ok(value) => {
                self.execute_batch(operation, "COMMIT")?;
                Ok(value)
            }
            Err(error) => {
                let _ = self.execute_batch(operation, "ROLLBACK");
                Err(error)
            }
        }
    }

    pub(super) fn execute_batch(
        &self,
        operation: &'static str,
        sql: &str,
    ) -> Result<(), QianjiBpmnDataStoreError> {
        self.connection()
            .execute_batch(sql)
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation,
                message: error.to_string(),
            })
    }

    fn ensure_schema(&self) -> Result<(), QianjiBpmnDataStoreError> {
        self.connection()
            .execute_batch(&format!(
                r"
CREATE TABLE IF NOT EXISTS {WORKFLOW_DATA_TABLE} (
    instance_id TEXT NOT NULL,
    record_key TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    updated_at_ms BIGINT NOT NULL,
    PRIMARY KEY (instance_id, record_key)
);
"
            ))
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "ensure_workflow_data_schema",
                message: error.to_string(),
            })
    }
}

fn validate_record(record: &QianjiBpmnDataRecord) -> Result<(), QianjiBpmnDataStoreError> {
    validate_field("instance_id", &record.instance_id)?;
    validate_field("record_key", &record.record_key)
}

pub(super) fn validate_field(
    field: &'static str,
    value: &str,
) -> Result<(), QianjiBpmnDataStoreError> {
    if value.trim().is_empty() {
        return Err(QianjiBpmnDataStoreError::BlankField { field });
    }
    Ok(())
}

pub(super) fn timestamp_to_i64(
    record_key: &str,
    updated_at_ms: u64,
) -> Result<i64, QianjiBpmnDataStoreError> {
    i64::try_from(updated_at_ms).map_err(|_| QianjiBpmnDataStoreError::TimestampOutOfRange {
        record_key: record_key.to_string(),
        updated_at_ms,
    })
}

fn decode_record(
    instance_id: &str,
    record_key: &str,
    payload_json: &str,
    updated_at_ms: i64,
) -> Result<QianjiBpmnDataRecord, QianjiBpmnDataStoreError> {
    let payload =
        serde_json::from_str(payload_json).map_err(|error| QianjiBpmnDataStoreError::Codec {
            operation: "decode_workflow_data_payload",
            message: error.to_string(),
        })?;
    let updated_at_ms =
        u64::try_from(updated_at_ms).map_err(|error| QianjiBpmnDataStoreError::Storage {
            operation: "decode_workflow_data_timestamp",
            message: error.to_string(),
        })?;
    Ok(QianjiBpmnDataRecord::new(
        instance_id,
        record_key,
        payload,
        updated_at_ms,
    ))
}

struct WorkflowStateRecordParts {
    instance_id: String,
    payload_json: String,
    updated_at_ms: i64,
}

fn workflow_state_record_parts(
    checkpoint: &BpmnCheckpointEnvelope,
) -> Result<WorkflowStateRecordParts, QianjiBpmnDataStoreError> {
    let instance_id = checkpoint.state.instance_id.as_ref();
    validate_field("instance_id", instance_id)?;
    let payload_json =
        serde_json::to_string(checkpoint).map_err(|error| QianjiBpmnDataStoreError::Codec {
            operation: "serialize_workflow_state_snapshot",
            message: error.to_string(),
        })?;
    let updated_at_ms = timestamp_to_i64(
        QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY,
        checkpoint.state.updated_at_ms,
    )?;
    Ok(WorkflowStateRecordParts {
        instance_id: instance_id.to_string(),
        payload_json,
        updated_at_ms,
    })
}
