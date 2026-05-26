use std::collections::HashMap;
use std::sync::Arc;

use crate::arrow_schema::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaContractError, ArrowSchemaDataType,
    WENDAO_TABLE_METADATA_KEY, build_arrow_schema, validate_record_batch_schema,
};
use crate::duckdb_crate::OptionalExt;
use arrow::{
    array::{Int64Array, StringArray},
    record_batch::RecordBatch,
};
use xiuxian_qianji_bpmn_engine::BpmnCheckpointEnvelope;

use super::store::{timestamp_to_i64, validate_field};
use super::{
    QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY, QianjiBpmnDataStoreError, QianjiBpmnDuckDbDataStore,
    QianjiBpmnInstanceId,
};

const WORKFLOW_STATE_LOG_TABLE: &str = "qianji_bpmn_workflow_state_events";
const WORKFLOW_STATE_SNAPSHOT_ARROW_COLUMNS: [ArrowSchemaColumn; 4] = [
    ArrowSchemaColumn::new("instance_id", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("sequence", ArrowSchemaDataType::Int64),
    ArrowSchemaColumn::new("updated_at_ms", ArrowSchemaDataType::Int64),
    ArrowSchemaColumn::new("payload_json", ArrowSchemaDataType::Utf8),
];
const LOAD_LATEST_WORKFLOW_STATE_EVENT_SQL: &str = "
SELECT payload_json
FROM qianji_bpmn_workflow_state_events
WHERE instance_id = ?1
ORDER BY sequence DESC, updated_at_ms DESC
LIMIT 1";
const LOAD_LATEST_WORKFLOW_STATE_TABLE_SQL: &str = "
SELECT payload_json
FROM qianji_bpmn_workflow_state_latest
WHERE instance_id = ?1";
const LOAD_COMPACTED_WORKFLOW_STATE_SNAPSHOTS_SQL: &str = "
SELECT payload_json
FROM qianji_bpmn_workflow_state_latest
ORDER BY updated_at_ms DESC, sequence DESC, instance_id ASC";
const COMPACT_LATEST_WORKFLOW_STATE_SNAPSHOTS_SQL: &str = "
INSERT INTO qianji_bpmn_workflow_state_latest
    (instance_id, sequence, updated_at_ms, payload_json)
SELECT instance_id, sequence, updated_at_ms, payload_json
FROM (
    SELECT
        instance_id,
        sequence,
        updated_at_ms,
        payload_json,
        row_number() OVER (
            PARTITION BY instance_id
            ORDER BY sequence DESC, updated_at_ms DESC
        ) AS row_rank
    FROM qianji_bpmn_workflow_state_events
)
WHERE row_rank = 1
ON CONFLICT(instance_id) DO UPDATE
SET sequence = excluded.sequence,
    updated_at_ms = excluded.updated_at_ms,
    payload_json = excluded.payload_json
WHERE excluded.sequence > qianji_bpmn_workflow_state_latest.sequence
   OR (
       excluded.sequence = qianji_bpmn_workflow_state_latest.sequence
       AND excluded.updated_at_ms >= qianji_bpmn_workflow_state_latest.updated_at_ms
   )";
const APPEND_WORKFLOW_STATE_SNAPSHOT_SQL: &str = "
INSERT INTO qianji_bpmn_workflow_state_events
    (instance_id, sequence, updated_at_ms, payload_json)
VALUES (?1, ?2, ?3, ?4)";
const UPSERT_LATEST_WORKFLOW_STATE_SNAPSHOT_SQL: &str = "
INSERT INTO qianji_bpmn_workflow_state_latest
    (instance_id, sequence, updated_at_ms, payload_json)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT(instance_id) DO UPDATE
SET sequence = excluded.sequence,
    updated_at_ms = excluded.updated_at_ms,
    payload_json = excluded.payload_json
WHERE excluded.sequence > qianji_bpmn_workflow_state_latest.sequence
   OR (
       excluded.sequence = qianji_bpmn_workflow_state_latest.sequence
       AND excluded.updated_at_ms >= qianji_bpmn_workflow_state_latest.updated_at_ms
   )";
const DELETE_WORKFLOW_STATE_SNAPSHOTS_SQL: &str = "
DELETE FROM qianji_bpmn_workflow_state_events
WHERE instance_id = ?1";
const DELETE_LATEST_WORKFLOW_STATE_SNAPSHOT_SQL: &str = "
DELETE FROM qianji_bpmn_workflow_state_latest
WHERE instance_id = ?1";
const COUNT_WORKFLOW_STATE_SNAPSHOTS_SQL: &str = "
SELECT COUNT(*)
FROM qianji_bpmn_workflow_state_events
WHERE instance_id = ?1";

impl QianjiBpmnDuckDbDataStore {
    /// Appends one checkpoint snapshot to the workflow-state event log.
    ///
    /// This is the preferred hot path for local no-server checkpoint saves:
    /// it avoids primary-key conflict maintenance and lets `DuckDB` do what it
    /// is good at, append durable columnar records.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when the snapshot cannot be encoded
    /// or when `DuckDB` rejects the append.
    pub fn append_workflow_state_snapshot(
        &self,
        checkpoint: &BpmnCheckpointEnvelope,
    ) -> Result<(), QianjiBpmnDataStoreError> {
        let instance_id = checkpoint.state.instance_id.as_ref();
        validate_field("instance_id", instance_id)?;
        let sequence = sequence_to_i64(checkpoint.sequence)?;
        let updated_at_ms = timestamp_to_i64(
            QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY,
            checkpoint.state.updated_at_ms,
        )?;
        let payload_json = serialize_checkpoint(checkpoint)?;
        let mut statement = self
            .connection()
            .prepare_cached(APPEND_WORKFLOW_STATE_SNAPSHOT_SQL)
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "prepare_append_workflow_state_snapshot",
                message: error.to_string(),
            })?;
        statement
            .execute(crate::duckdb_crate::params![
                instance_id,
                sequence,
                updated_at_ms,
                payload_json,
            ])
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "append_workflow_state_snapshot",
                message: error.to_string(),
            })?;
        self.cache_latest_workflow_state_snapshot(checkpoint)?;
        Ok(())
    }

    /// Appends checkpoint snapshots through `DuckDB`'s Arrow appender path.
    ///
    /// The batch path is append-only by design. Latest-state reads are resolved
    /// from the event log by `(sequence, updated_at_ms)`, which avoids paying a
    /// synchronous row-upsert cost for every checkpoint batch.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when a snapshot cannot be encoded
    /// into the Arrow batch shape or when `DuckDB` rejects the append.
    pub fn append_workflow_state_snapshots<'a>(
        &self,
        checkpoints: impl IntoIterator<Item = &'a BpmnCheckpointEnvelope>,
    ) -> Result<usize, QianjiBpmnDataStoreError> {
        let checkpoints = checkpoints.into_iter().collect::<Vec<_>>();
        let (batch, count) = workflow_state_snapshots_to_batch(checkpoints.iter().copied())?;
        if count == 0 {
            return Ok(0);
        }
        self.execute_in_transaction("append_workflow_state_snapshot_batch", || {
            self.append_snapshot_batch(WORKFLOW_STATE_LOG_TABLE, batch)?;
            Ok(count)
        })?;
        for checkpoint in checkpoints {
            self.cache_latest_workflow_state_snapshot(checkpoint)?;
        }
        Ok(count)
    }

    /// Rebuilds the compacted latest-checkpoint table from the append log.
    ///
    /// This is a cold recovery optimization. It keeps checkpoint saves
    /// append-only and pays the materialization cost once when a local `DuckDB`
    /// workflow-state store is first used for resume/status access.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when `DuckDB` rejects the
    /// compaction query.
    pub fn compact_workflow_state_latest_snapshots(&self) -> Result<(), QianjiBpmnDataStoreError> {
        self.execute_in_transaction("compact_workflow_state_latest_snapshots", || {
            self.execute_batch(
                "compact_workflow_state_latest_snapshots",
                COMPACT_LATEST_WORKFLOW_STATE_SNAPSHOTS_SQL,
            )
        })
    }

    /// Loads one checkpoint from the compacted latest-checkpoint table.
    ///
    /// This method is intended for cold recovery after
    /// [`Self::compact_workflow_state_latest_snapshots`] has rebuilt the latest
    /// table. Use [`Self::load_latest_workflow_state_snapshot`] when callers
    /// need to read directly from the append log without relying on compaction.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when the lookup fails or the stored
    /// JSON payload cannot be decoded into a checkpoint envelope.
    pub fn load_compacted_workflow_state_snapshot(
        &self,
        instance_id: impl Into<QianjiBpmnInstanceId>,
    ) -> Result<Option<BpmnCheckpointEnvelope>, QianjiBpmnDataStoreError> {
        let instance_id = instance_id.into();
        validate_field("instance_id", instance_id.as_str())?;
        self.load_latest_workflow_state_table_payload(instance_id.as_str())?
            .map(|payload_json| decode_checkpoint(&payload_json))
            .transpose()
    }

    /// Loads all compacted latest checkpoint snapshots.
    ///
    /// This is intended for cold cache hydration after
    /// [`Self::compact_workflow_state_latest_snapshots`] has rebuilt the latest
    /// table.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when the lookup fails or any stored
    /// JSON payload cannot be decoded into a checkpoint envelope.
    pub fn load_compacted_workflow_state_snapshots(
        &self,
    ) -> Result<Vec<BpmnCheckpointEnvelope>, QianjiBpmnDataStoreError> {
        let mut statement = self
            .connection()
            .prepare_cached(LOAD_COMPACTED_WORKFLOW_STATE_SNAPSHOTS_SQL)
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "prepare_load_compacted_workflow_state_snapshots",
                message: error.to_string(),
            })?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "load_compacted_workflow_state_snapshots",
                message: error.to_string(),
            })?;
        let mut checkpoints = Vec::new();
        for row in rows {
            let payload_json = row.map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "read_compacted_workflow_state_snapshot",
                message: error.to_string(),
            })?;
            checkpoints.push(decode_checkpoint(&payload_json)?);
        }
        Ok(checkpoints)
    }

    /// Loads the latest checkpoint snapshot for one BPMN instance.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when the lookup fails or the stored
    /// JSON payload cannot be decoded into a checkpoint envelope.
    pub fn load_latest_workflow_state_snapshot(
        &self,
        instance_id: impl Into<QianjiBpmnInstanceId>,
    ) -> Result<Option<BpmnCheckpointEnvelope>, QianjiBpmnDataStoreError> {
        let instance_id = instance_id.into();
        validate_field("instance_id", instance_id.as_str())?;
        if let Some(checkpoint) =
            self.cached_latest_workflow_state_snapshot(instance_id.as_str())?
        {
            return Ok(Some(checkpoint));
        }
        let mut statement = self
            .connection()
            .prepare_cached(LOAD_LATEST_WORKFLOW_STATE_EVENT_SQL)
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "prepare_load_latest_workflow_state_snapshot",
                message: error.to_string(),
            })?;
        let payload_json: Option<String> = statement
            .query_row(crate::duckdb_crate::params![instance_id.as_str()], |row| {
                row.get(0)
            })
            .optional()
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "load_latest_workflow_state_snapshot",
                message: error.to_string(),
            })?;
        let payload_json = match payload_json {
            Some(payload_json) => Some(payload_json),
            None => self.load_latest_workflow_state_table_payload(instance_id.as_str())?,
        };
        let checkpoint = payload_json
            .map(|payload_json| decode_checkpoint(&payload_json))
            .transpose()?;
        if let Some(checkpoint) = checkpoint.as_ref() {
            self.cache_latest_workflow_state_snapshot(checkpoint)?;
        }
        Ok(checkpoint)
    }

    /// Upserts one latest checkpoint snapshot through the dedicated
    /// workflow-state table.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when the snapshot cannot be encoded
    /// or when `DuckDB` rejects the point upsert.
    pub fn upsert_latest_workflow_state_snapshot(
        &self,
        checkpoint: &BpmnCheckpointEnvelope,
    ) -> Result<(), QianjiBpmnDataStoreError> {
        let instance_id = checkpoint.state.instance_id.as_ref();
        validate_field("instance_id", instance_id)?;
        let sequence = sequence_to_i64(checkpoint.sequence)?;
        let updated_at_ms = timestamp_to_i64(
            QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY,
            checkpoint.state.updated_at_ms,
        )?;
        let payload_json = serialize_checkpoint(checkpoint)?;
        let mut statement = self
            .connection()
            .prepare_cached(UPSERT_LATEST_WORKFLOW_STATE_SNAPSHOT_SQL)
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "prepare_upsert_latest_workflow_state_snapshot",
                message: error.to_string(),
            })?;
        statement
            .execute(crate::duckdb_crate::params![
                instance_id,
                sequence,
                updated_at_ms,
                payload_json,
            ])
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "upsert_latest_workflow_state_snapshot",
                message: error.to_string(),
            })?;
        self.cache_latest_workflow_state_snapshot(checkpoint)?;
        Ok(())
    }

    /// Deletes all append-log checkpoint snapshots for one BPMN instance.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when the instance id is invalid or
    /// `DuckDB` rejects the delete.
    pub fn delete_workflow_state_snapshots(
        &self,
        instance_id: impl Into<QianjiBpmnInstanceId>,
    ) -> Result<bool, QianjiBpmnDataStoreError> {
        let instance_id = instance_id.into();
        validate_field("instance_id", instance_id.as_str())?;
        self.execute_in_transaction("delete_workflow_state_snapshots", || {
            let deleted_events = self.delete_rows_by_instance_id(
                DELETE_WORKFLOW_STATE_SNAPSHOTS_SQL,
                "prepare_delete_workflow_state_snapshots",
                "delete_workflow_state_snapshots",
                instance_id.as_str(),
            )?;
            let deleted_latest = self.delete_rows_by_instance_id(
                DELETE_LATEST_WORKFLOW_STATE_SNAPSHOT_SQL,
                "prepare_delete_latest_workflow_state_snapshot",
                "delete_latest_workflow_state_snapshot",
                instance_id.as_str(),
            )?;
            self.remove_cached_latest_workflow_state_snapshot(instance_id.as_str())?;
            Ok(deleted_events || deleted_latest)
        })
    }

    /// Counts append-log checkpoint snapshots for one BPMN instance.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiBpmnDataStoreError`] when the instance id is invalid or
    /// `DuckDB` rejects the count query.
    pub fn workflow_state_snapshot_count(
        &self,
        instance_id: impl Into<QianjiBpmnInstanceId>,
    ) -> Result<u64, QianjiBpmnDataStoreError> {
        let instance_id = instance_id.into();
        validate_field("instance_id", instance_id.as_str())?;
        let mut statement = self
            .connection()
            .prepare_cached(COUNT_WORKFLOW_STATE_SNAPSHOTS_SQL)
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "prepare_count_workflow_state_snapshots",
                message: error.to_string(),
            })?;
        let count: i64 = statement
            .query_row(crate::duckdb_crate::params![instance_id.as_str()], |row| {
                row.get(0)
            })
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "count_workflow_state_snapshots",
                message: error.to_string(),
            })?;
        u64::try_from(count).map_err(|error| QianjiBpmnDataStoreError::Storage {
            operation: "decode_workflow_state_snapshot_count",
            message: error.to_string(),
        })
    }

    pub(super) fn ensure_workflow_state_log_schema(&self) -> Result<(), QianjiBpmnDataStoreError> {
        self.connection()
            .execute_batch(
                r"
CREATE TABLE IF NOT EXISTS qianji_bpmn_workflow_state_events (
    instance_id TEXT NOT NULL,
    sequence BIGINT NOT NULL,
    updated_at_ms BIGINT NOT NULL,
    payload_json TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS qianji_bpmn_workflow_state_events_latest_idx
ON qianji_bpmn_workflow_state_events(instance_id, sequence, updated_at_ms);
CREATE TABLE IF NOT EXISTS qianji_bpmn_workflow_state_latest (
    instance_id TEXT PRIMARY KEY,
    sequence BIGINT NOT NULL,
    updated_at_ms BIGINT NOT NULL,
    payload_json TEXT NOT NULL
);
",
            )
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "ensure_workflow_state_log_schema",
                message: error.to_string(),
            })
    }

    fn append_snapshot_batch(
        &self,
        table_name: &str,
        batch: RecordBatch,
    ) -> Result<(), QianjiBpmnDataStoreError> {
        let mut appender = self.connection().appender(table_name).map_err(|error| {
            QianjiBpmnDataStoreError::Storage {
                operation: "open_workflow_state_snapshot_appender",
                message: error.to_string(),
            }
        })?;
        appender
            .append_record_batch(batch)
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "append_workflow_state_snapshot_batch",
                message: error.to_string(),
            })?;
        appender
            .flush()
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "flush_workflow_state_snapshot_appender",
                message: error.to_string(),
            })
    }

    fn load_latest_workflow_state_table_payload(
        &self,
        instance_id: &str,
    ) -> Result<Option<String>, QianjiBpmnDataStoreError> {
        let mut statement = self
            .connection()
            .prepare_cached(LOAD_LATEST_WORKFLOW_STATE_TABLE_SQL)
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "prepare_load_latest_workflow_state_table_snapshot",
                message: error.to_string(),
            })?;
        statement
            .query_row(crate::duckdb_crate::params![instance_id], |row| row.get(0))
            .optional()
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: "load_latest_workflow_state_table_snapshot",
                message: error.to_string(),
            })
    }

    fn delete_rows_by_instance_id(
        &self,
        sql: &str,
        prepare_operation: &'static str,
        execute_operation: &'static str,
        instance_id: &str,
    ) -> Result<bool, QianjiBpmnDataStoreError> {
        let mut statement = self.connection().prepare_cached(sql).map_err(|error| {
            QianjiBpmnDataStoreError::Storage {
                operation: prepare_operation,
                message: error.to_string(),
            }
        })?;
        let changed = statement
            .execute(crate::duckdb_crate::params![instance_id])
            .map_err(|error| QianjiBpmnDataStoreError::Storage {
                operation: execute_operation,
                message: error.to_string(),
            })?;
        Ok(changed > 0)
    }
}

fn workflow_state_snapshots_to_batch<'a>(
    checkpoints: impl IntoIterator<Item = &'a BpmnCheckpointEnvelope>,
) -> Result<(RecordBatch, usize), QianjiBpmnDataStoreError> {
    let columns = checkpoints
        .into_iter()
        .map(workflow_state_snapshot_columns)
        .collect::<Result<Vec<_>, QianjiBpmnDataStoreError>>()?;
    let count = columns.len();
    let contract = workflow_state_snapshot_arrow_contract();
    let batch = RecordBatch::try_new(
        workflow_state_snapshot_arrow_schema(&contract),
        vec![
            Arc::new(StringArray::from(
                columns
                    .iter()
                    .map(|column| column.instance_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Int64Array::from(
                columns
                    .iter()
                    .map(|column| column.sequence)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Int64Array::from(
                columns
                    .iter()
                    .map(|column| column.updated_at_ms)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                columns
                    .iter()
                    .map(|column| column.payload_json.as_str())
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(|error| QianjiBpmnDataStoreError::Storage {
        operation: "build_workflow_state_snapshot_arrow_batch",
        message: error.to_string(),
    })?;
    validate_record_batch_schema(&batch, &contract)
        .map_err(|error| workflow_state_schema_error(&error))?;
    Ok((batch, count))
}

fn workflow_state_snapshot_arrow_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        WORKFLOW_STATE_LOG_TABLE,
        true,
        WORKFLOW_STATE_SNAPSHOT_ARROW_COLUMNS.to_vec(),
    )
}

fn workflow_state_snapshot_arrow_schema(
    contract: &ArrowSchemaContract,
) -> Arc<arrow::datatypes::Schema> {
    Arc::new(build_arrow_schema(
        contract,
        HashMap::from([(
            WENDAO_TABLE_METADATA_KEY.to_string(),
            WORKFLOW_STATE_LOG_TABLE.to_string(),
        )]),
    ))
}

fn workflow_state_schema_error(error: &ArrowSchemaContractError) -> QianjiBpmnDataStoreError {
    QianjiBpmnDataStoreError::Storage {
        operation: "validate_workflow_state_snapshot_arrow_schema",
        message: error.to_string(),
    }
}

struct WorkflowStateSnapshotColumns {
    instance_id: String,
    sequence: i64,
    updated_at_ms: i64,
    payload_json: String,
}

fn workflow_state_snapshot_columns(
    checkpoint: &BpmnCheckpointEnvelope,
) -> Result<WorkflowStateSnapshotColumns, QianjiBpmnDataStoreError> {
    let instance_id = checkpoint.state.instance_id.as_ref();
    validate_field("instance_id", instance_id)?;
    Ok(WorkflowStateSnapshotColumns {
        instance_id: instance_id.to_string(),
        sequence: sequence_to_i64(checkpoint.sequence)?,
        updated_at_ms: timestamp_to_i64(
            QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY,
            checkpoint.state.updated_at_ms,
        )?,
        payload_json: serialize_checkpoint(checkpoint)?,
    })
}

fn sequence_to_i64(sequence: u64) -> Result<i64, QianjiBpmnDataStoreError> {
    i64::try_from(sequence).map_err(|_| QianjiBpmnDataStoreError::Storage {
        operation: "encode_workflow_state_snapshot_sequence",
        message: format!("checkpoint sequence {sequence} is out of range"),
    })
}

fn serialize_checkpoint(
    checkpoint: &BpmnCheckpointEnvelope,
) -> Result<String, QianjiBpmnDataStoreError> {
    serde_json::to_string(checkpoint).map_err(|error| QianjiBpmnDataStoreError::Codec {
        operation: "serialize_workflow_state_snapshot",
        message: error.to_string(),
    })
}

fn decode_checkpoint(
    payload_json: &str,
) -> Result<BpmnCheckpointEnvelope, QianjiBpmnDataStoreError> {
    serde_json::from_str(payload_json).map_err(|error| QianjiBpmnDataStoreError::Codec {
        operation: "decode_workflow_state_snapshot",
        message: error.to_string(),
    })
}
