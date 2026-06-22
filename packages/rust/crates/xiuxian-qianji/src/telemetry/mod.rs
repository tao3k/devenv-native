//! Swarm pulse telemetry rooted in `api`; event contracts and emitters stay private.

#[path = "../telemetry_api.rs"]
mod api;
#[path = "../telemetry_events.rs"]
mod events;
#[path = "../telemetry_traits.rs"]
mod traits;
#[cfg(feature = "valkey")]
#[path = "../telemetry_valkey.rs"]
mod valkey;

#[cfg(feature = "valkey")]
pub use api::ValkeyPulseEmitter;
pub use api::{
    CognitiveDistributionMetrics, ConsensusStatus, DEFAULT_PULSE_CHANNEL, NodeTransitionPhase,
    NoopPulseEmitter, PulseEmitter, SwarmEvent, unix_millis_now,
};
