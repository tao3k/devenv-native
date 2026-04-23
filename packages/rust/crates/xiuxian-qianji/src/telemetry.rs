//! Swarm pulse telemetry rooted in `api`; event contracts and emitters stay private.

#[path = "telemetry_api.rs"]
mod api;
#[path = "telemetry_events.rs"]
mod events;
#[path = "telemetry_traits.rs"]
mod traits;
#[path = "telemetry_valkey.rs"]
mod valkey;

pub use api::{
    CognitiveDistributionMetrics, ConsensusStatus, DEFAULT_PULSE_CHANNEL, NodeTransitionPhase,
    NoopPulseEmitter, PulseEmitter, SwarmEvent, ValkeyPulseEmitter, unix_millis_now,
};
