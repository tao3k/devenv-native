//! Start in `alerts`; it owns telemetry emission helpers.

mod alerts;
#[cfg(test)]
#[path = "../../../../tests/unit/scheduler/core/telemetry/node_transition.rs"]
mod node_transition;
