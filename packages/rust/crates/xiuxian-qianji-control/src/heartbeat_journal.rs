//! Worker heartbeat journal recording helpers.

use crate::{
    ControlError, ControlEvent, ControlEventKind, ControlEventRecord, ControlLedger, ControlResult,
    HotStateStore, RunId, WorkerHeartbeat,
};

/// Named request for recording one Worker heartbeat fact.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerHeartbeatJournalRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Worker heartbeat payload.
    pub heartbeat: WorkerHeartbeat,
}

impl WorkerHeartbeatJournalRecord {
    /// Creates a Worker heartbeat journal record request.
    #[must_use]
    pub const fn new(run_id: RunId, heartbeat: WorkerHeartbeat) -> Self {
        Self { run_id, heartbeat }
    }
}

/// Records one Worker heartbeat as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the heartbeat TTL is invalid or the ledger
/// append fails.
pub fn record_worker_heartbeat<L>(
    ledger: &L,
    request: WorkerHeartbeatJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    validate_worker_heartbeat(&request.heartbeat)?;
    let WorkerHeartbeatJournalRecord { run_id, heartbeat } = request;
    let observed_at_ms = heartbeat.observed_at_ms;
    ledger.append_event(ControlEvent::run(
        run_id,
        observed_at_ms,
        ControlEventKind::WorkerHeartbeatObserved { heartbeat },
    ))
}

/// Records one Worker heartbeat in hot state and durable history.
///
/// # Errors
///
/// Returns a control error when the heartbeat TTL is invalid, hot-state write
/// fails, or durable ledger append fails. The hot-state write happens before
/// the durable append so a durable heartbeat fact represents a successful
/// liveness mirror.
pub async fn record_worker_heartbeat_with_hot_state<L, H>(
    ledger: &L,
    hot_state: &H,
    request: WorkerHeartbeatJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
    H: HotStateStore + ?Sized,
{
    validate_worker_heartbeat(&request.heartbeat)?;
    hot_state.heartbeat(request.heartbeat.clone()).await?;
    record_worker_heartbeat(ledger, request)
}

fn validate_worker_heartbeat(heartbeat: &WorkerHeartbeat) -> ControlResult<()> {
    if heartbeat.expires_at_ms > heartbeat.observed_at_ms {
        return Ok(());
    }
    Err(ControlError::Storage {
        operation: "validate_worker_heartbeat_ttl",
        message: "expires_at_ms must be greater than observed_at_ms".to_string(),
    })
}
