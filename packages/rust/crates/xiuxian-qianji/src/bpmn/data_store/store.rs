use xiuxian_db_store::duckdb::DuckDbConnection;
use xiuxian_db_store::duckdb_crate::OptionalExt;

use super::{QianjiBpmnDataRecord, QianjiBpmnDataStoreError, QianjiBpmnDuckDbDataStoreConfig};

const WORKFLOW_DATA_TABLE: &str = "qianji_bpmn_workflow_data_records";

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
        let updated_at_ms = timestamp_to_i64(record)?;
        self.connection()
            .execute(
                &format!(
                    "INSERT INTO {WORKFLOW_DATA_TABLE}
                     (instance_id, record_key, payload_json, updated_at_ms)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(instance_id, record_key) DO UPDATE
                     SET payload_json = excluded.payload_json,
                         updated_at_ms = excluded.updated_at_ms"
                ),
                xiuxian_db_store::duckdb_crate::params![
                    record.instance_id,
                    record.record_key,
                    payload_json,
                    updated_at_ms,
                ],
            )
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
        let row: Option<(String, i64)> = self
            .connection()
            .query_row(
                &format!(
                    "SELECT payload_json, updated_at_ms
                     FROM {WORKFLOW_DATA_TABLE}
                     WHERE instance_id = ?1 AND record_key = ?2"
                ),
                xiuxian_db_store::duckdb_crate::params![instance_id, record_key],
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
        let changed = self
            .connection()
            .execute(
                &format!(
                    "DELETE FROM {WORKFLOW_DATA_TABLE}
                     WHERE instance_id = ?1 AND record_key = ?2"
                ),
                xiuxian_db_store::duckdb_crate::params![instance_id, record_key],
            )
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "delete_workflow_data_record",
                message: error.to_string(),
            })?;
        Ok(changed > 0)
    }

    fn connection(&self) -> &xiuxian_db_store::duckdb_crate::Connection {
        self.connection.connection()
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

fn validate_field(field: &'static str, value: &str) -> Result<(), QianjiBpmnDataStoreError> {
    if value.trim().is_empty() {
        return Err(QianjiBpmnDataStoreError::BlankField { field });
    }
    Ok(())
}

fn timestamp_to_i64(record: &QianjiBpmnDataRecord) -> Result<i64, QianjiBpmnDataStoreError> {
    i64::try_from(record.updated_at_ms).map_err(|_| QianjiBpmnDataStoreError::TimestampOutOfRange {
        record_key: record.record_key.clone(),
        updated_at_ms: record.updated_at_ms,
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
