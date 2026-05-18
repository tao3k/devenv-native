//! DuckDB-backed append-only control event ledger.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use crate::{
    ControlError, ControlEvent, ControlEventKind, ControlEventRecord, ControlLedger, ControlResult,
    RunId,
};

const ENSURE_SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS qianji_control_events (
    sequence BIGINT NOT NULL,
    run_id TEXT NOT NULL,
    step_id TEXT,
    occurred_at_ms BIGINT NOT NULL,
    event_kind TEXT NOT NULL,
    event_json TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS qianji_control_events_sequence_idx
ON qianji_control_events(sequence);
CREATE INDEX IF NOT EXISTS qianji_control_events_run_sequence_idx
ON qianji_control_events(run_id, sequence);
";
const NEXT_SEQUENCE_SQL: &str = "
SELECT COALESCE(MAX(sequence), 0) + 1
FROM qianji_control_events";
const APPEND_EVENT_SQL: &str = "
INSERT INTO qianji_control_events
    (sequence, run_id, step_id, occurred_at_ms, event_kind, event_json)
VALUES (?1, ?2, ?3, ?4, ?5, ?6)";
const LOAD_EVENTS_SQL: &str = "
SELECT sequence, event_json
FROM qianji_control_events
WHERE run_id = ?1
ORDER BY sequence ASC";

/// DuckDB-backed append-only control event ledger.
///
/// This adapter is durable event storage only. It does not own queues,
/// leases, heartbeats, or latest-state truth tables.
pub struct DuckDbControlLedger {
    connection: Mutex<duckdb::Connection>,
}

impl DuckDbControlLedger {
    /// Opens a `DuckDB` ledger at a filesystem path.
    ///
    /// # Errors
    ///
    /// Returns [`ControlError`] when the parent directory cannot be created,
    /// `DuckDB` cannot open the database, or schema initialization fails.
    pub fn open(path: impl Into<PathBuf>) -> ControlResult<Self> {
        let path = path.into();
        ensure_parent_dir(&path)?;
        let connection = duckdb::Connection::open(path).map_err(|error| ControlError::Storage {
            operation: "open_duckdb_control_ledger",
            message: error.to_string(),
        })?;
        Self::from_connection(connection)
    }

    /// Opens an in-memory `DuckDB` ledger.
    ///
    /// # Errors
    ///
    /// Returns [`ControlError`] when `DuckDB` cannot open or initialize the
    /// in-memory database.
    pub fn open_in_memory() -> ControlResult<Self> {
        let connection =
            duckdb::Connection::open_in_memory().map_err(|error| ControlError::Storage {
                operation: "open_in_memory_duckdb_control_ledger",
                message: error.to_string(),
            })?;
        Self::from_connection(connection)
    }

    fn from_connection(connection: duckdb::Connection) -> ControlResult<Self> {
        let ledger = Self {
            connection: Mutex::new(connection),
        };
        ledger.ensure_schema()?;
        Ok(ledger)
    }

    fn ensure_schema(&self) -> ControlResult<()> {
        self.with_connection("ensure_duckdb_control_ledger_schema", |connection| {
            connection
                .execute_batch(ENSURE_SCHEMA_SQL)
                .map_err(|error| ControlError::Storage {
                    operation: "ensure_duckdb_control_ledger_schema",
                    message: error.to_string(),
                })
        })
    }

    fn with_connection<T>(
        &self,
        lock_name: &'static str,
        action: impl FnOnce(&duckdb::Connection) -> ControlResult<T>,
    ) -> ControlResult<T> {
        let connection = lock_connection(&self.connection, lock_name)?;
        action(&connection)
    }
}

impl ControlLedger for DuckDbControlLedger {
    fn append_event(&self, event: ControlEvent) -> ControlResult<ControlEventRecord> {
        self.with_connection("duckdb_control_ledger", |connection| {
            execute_batch(
                connection,
                "begin_duckdb_control_event_append",
                "BEGIN TRANSACTION",
            )?;
            let append_result = append_event_in_transaction(connection, event);
            match append_result {
                Ok(record) => {
                    execute_batch(connection, "commit_duckdb_control_event_append", "COMMIT")?;
                    Ok(record)
                }
                Err(error) => {
                    let _ = execute_batch(
                        connection,
                        "rollback_duckdb_control_event_append",
                        "ROLLBACK",
                    );
                    Err(error)
                }
            }
        })
    }

    fn load_events(&self, run_id: &RunId) -> ControlResult<Vec<ControlEventRecord>> {
        self.with_connection("duckdb_control_ledger", |connection| {
            let mut statement = connection
                .prepare_cached(LOAD_EVENTS_SQL)
                .map_err(|error| ControlError::Storage {
                    operation: "prepare_load_duckdb_control_events",
                    message: error.to_string(),
                })?;
            let rows = statement
                .query_map(duckdb::params![run_id.as_str()], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|error| ControlError::Storage {
                    operation: "load_duckdb_control_events",
                    message: error.to_string(),
                })?;
            rows.map(|row| {
                let (sequence, event_json) = row.map_err(|error| ControlError::Storage {
                    operation: "read_duckdb_control_event_row",
                    message: error.to_string(),
                })?;
                let event =
                    serde_json::from_str(&event_json).map_err(|error| ControlError::Codec {
                        operation: "decode_duckdb_control_event",
                        message: error.to_string(),
                    })?;
                Ok(ControlEventRecord {
                    sequence: decode_sequence(sequence)?,
                    event,
                })
            })
            .collect()
        })
    }
}

fn append_event_in_transaction(
    connection: &duckdb::Connection,
    event: ControlEvent,
) -> ControlResult<ControlEventRecord> {
    let sequence = next_sequence(connection)?;
    let event_json = serde_json::to_string(&event).map_err(|error| ControlError::Codec {
        operation: "encode_duckdb_control_event",
        message: error.to_string(),
    })?;
    let step_id = event.step_id.as_ref().map(crate::StepId::as_str);
    let occurred_at_ms =
        i64::try_from(event.occurred_at_ms).map_err(|error| ControlError::Storage {
            operation: "encode_duckdb_control_event_timestamp",
            message: error.to_string(),
        })?;
    let sequence_i64 = encode_sequence(sequence)?;
    let mut statement = connection
        .prepare_cached(APPEND_EVENT_SQL)
        .map_err(|error| ControlError::Storage {
            operation: "prepare_append_duckdb_control_event",
            message: error.to_string(),
        })?;
    statement
        .execute(duckdb::params![
            sequence_i64,
            event.run_id.as_str(),
            step_id,
            occurred_at_ms,
            event_kind_label(&event.kind),
            event_json,
        ])
        .map_err(|error| ControlError::Storage {
            operation: "append_duckdb_control_event",
            message: error.to_string(),
        })?;
    Ok(ControlEventRecord { sequence, event })
}

fn next_sequence(connection: &duckdb::Connection) -> ControlResult<u64> {
    let next: i64 = connection
        .query_row(NEXT_SEQUENCE_SQL, [], |row| row.get(0))
        .map_err(|error| ControlError::Storage {
            operation: "load_next_duckdb_control_event_sequence",
            message: error.to_string(),
        })?;
    decode_sequence(next)
}

fn encode_sequence(sequence: u64) -> ControlResult<i64> {
    i64::try_from(sequence).map_err(|error| ControlError::Storage {
        operation: "encode_duckdb_control_event_sequence",
        message: error.to_string(),
    })
}

fn decode_sequence(sequence: i64) -> ControlResult<u64> {
    u64::try_from(sequence).map_err(|error| ControlError::Storage {
        operation: "decode_duckdb_control_event_sequence",
        message: error.to_string(),
    })
}

fn execute_batch(
    connection: &duckdb::Connection,
    operation: &'static str,
    sql: &str,
) -> ControlResult<()> {
    connection
        .execute_batch(sql)
        .map_err(|error| ControlError::Storage {
            operation,
            message: error.to_string(),
        })
}

fn ensure_parent_dir(path: &Path) -> ControlResult<()> {
    let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return Ok(());
    };
    std::fs::create_dir_all(parent).map_err(|error| ControlError::Storage {
        operation: "create_duckdb_control_ledger_parent_dir",
        message: error.to_string(),
    })
}

fn lock_connection<'a>(
    mutex: &'a Mutex<duckdb::Connection>,
    lock_name: &'static str,
) -> ControlResult<MutexGuard<'a, duckdb::Connection>> {
    mutex.lock().map_err(|error| ControlError::LockPoisoned {
        lock_name,
        message: error.to_string(),
    })
}

fn event_kind_label(kind: &ControlEventKind) -> &'static str {
    match kind {
        ControlEventKind::RunCreated { .. } => "run_created",
        ControlEventKind::RunAdmitted => "run_admitted",
        ControlEventKind::PlanRecorded { .. } => "plan_recorded",
        ControlEventKind::StepCreated { .. } => "step_created",
        ControlEventKind::StepQueued => "step_queued",
        ControlEventKind::StepLeaseAcquired { .. } => "step_lease_acquired",
        ControlEventKind::StepLeaseRenewed { .. } => "step_lease_renewed",
        ControlEventKind::StepLeaseReleased { .. } => "step_lease_released",
        ControlEventKind::StepStarted => "step_started",
        ControlEventKind::StepWaiting { .. } => "step_waiting",
        ControlEventKind::ToolCallRecorded { .. } => "tool_call_recorded",
        ControlEventKind::AgentProposalRecorded { .. } => "agent_proposal_recorded",
        ControlEventKind::AgentDecisionRecorded { .. } => "agent_decision_recorded",
        ControlEventKind::ActivityScheduled { .. } => "activity_scheduled",
        ControlEventKind::ActivityStarted { .. } => "activity_started",
        ControlEventKind::ActivityCompleted { .. } => "activity_completed",
        ControlEventKind::ActivityFailed { .. } => "activity_failed",
        ControlEventKind::SignalReceived { .. } => "signal_received",
        ControlEventKind::TimerScheduled { .. } => "timer_scheduled",
        ControlEventKind::TimerFired { .. } => "timer_fired",
        ControlEventKind::VersionPinned { .. } => "version_pinned",
        ControlEventKind::ArtifactAttached { .. } => "artifact_attached",
        ControlEventKind::EvidenceAttached { .. } => "evidence_attached",
        ControlEventKind::CostObserved { .. } => "cost_observed",
        ControlEventKind::GateEvaluated { .. } => "gate_evaluated",
        ControlEventKind::RecoveryStarted { .. } => "recovery_started",
        ControlEventKind::WorkerHeartbeatObserved { .. } => "worker_heartbeat_observed",
        ControlEventKind::StepSucceeded => "step_succeeded",
        ControlEventKind::StepFailed { .. } => "step_failed",
        ControlEventKind::StepBlocked { .. } => "step_blocked",
        ControlEventKind::StepCancelled { .. } => "step_cancelled",
        ControlEventKind::RunCompleted => "run_completed",
        ControlEventKind::RunFailed { .. } => "run_failed",
        ControlEventKind::RunBlocked { .. } => "run_blocked",
        ControlEventKind::RunAborted { .. } => "run_aborted",
    }
}
