use super::SwarmEvent;
use async_trait::async_trait;

/// Async sink for swarm pulse telemetry.
#[async_trait]
pub trait PulseEmitter: Send + Sync + std::fmt::Debug {
    /// Emits one swarm event into the telemetry transport.
    ///
    /// # Errors
    ///
    /// Returns an error when the underlying transport is permanently unavailable.
    async fn emit_pulse(&self, event: SwarmEvent) -> Result<(), String>;
}

/// No-op emitter used when telemetry is disabled.
#[derive(Debug, Default)]
pub struct NoopPulseEmitter;

#[async_trait]
impl PulseEmitter for NoopPulseEmitter {
    async fn emit_pulse(&self, _event: SwarmEvent) -> Result<(), String> {
        Ok(())
    }
}
