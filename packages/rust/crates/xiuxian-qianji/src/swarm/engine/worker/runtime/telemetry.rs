use crate::swarm::engine_orchestrator::SwarmEngine;
use crate::telemetry::SwarmEvent;
use std::sync::Arc;

impl SwarmEngine {
    pub(in crate::swarm::engine::worker) fn emit_pulse_event(
        pulse_emitter: Option<&Arc<dyn crate::telemetry::PulseEmitter>>,
        event: SwarmEvent,
    ) {
        let Some(emitter) = pulse_emitter.cloned() else {
            return;
        };
        std::mem::drop(tokio::spawn(async move {
            if let Err(error) = emitter.emit_pulse(event).await {
                log::debug!("swarm telemetry emission skipped: {error}");
            }
        }));
    }
}
