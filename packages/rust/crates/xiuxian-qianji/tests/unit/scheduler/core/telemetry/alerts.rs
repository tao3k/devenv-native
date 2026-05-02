use crate::engine::QianjiEngine;
use crate::scheduler::core::QianjiScheduler;
use crate::telemetry::{ConsensusStatus, SwarmEvent, unix_millis_now};

impl QianjiScheduler {
    pub(in crate::scheduler::core) fn emit_consensus_spike(
        &self,
        session_id: &str,
        node_id: &str,
        status: ConsensusStatus,
        progress: Option<f32>,
        target: Option<f32>,
    ) {
        self.emit_event_non_blocking(SwarmEvent::ConsensusSpike {
            session_id: session_id.to_string(),
            node_id: node_id.to_string(),
            status,
            progress,
            target,
            timestamp_ms: unix_millis_now(),
        });
    }
}

#[test]
fn emit_consensus_spike_is_callable_without_emitter() {
    let scheduler = QianjiScheduler::new(QianjiEngine::default());
    scheduler.emit_consensus_spike(
        "session-1",
        "node-1",
        ConsensusStatus::Failed,
        Some(0.5),
        Some(0.8),
    );
}
