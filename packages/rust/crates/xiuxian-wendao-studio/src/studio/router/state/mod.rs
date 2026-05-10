//! Coordinates the Studio studio router state branch and keeps its child modules behind one documented reasoning-tree boundary.

mod cold_start;
mod graph;
mod lifecycle;
mod project_config;
mod search;
mod types;
mod ui;

#[cfg(test)]
#[path = "../../../../tests/unit/gateway/studio/router/state/mod.rs"]
mod tests;

pub use cold_start::{StudioSearchColdStartCorpusTelemetry, StudioSearchColdStartTelemetry};
#[cfg(test)]
pub(crate) use project_config::supported_code_kinds;
#[cfg(feature = "performance")]
pub(crate) use types::LocalCorpusScanCoalescingState;
#[cfg(any(test, feature = "performance"))]
pub(crate) use types::StudioSearchColdStartTelemetryState;
pub use types::{
    GatewayState, StudioBootstrapBackgroundIndexingTelemetry, StudioSearchColdStartEvent,
    StudioState,
};
#[cfg(test)]
pub(crate) use types::{GraphIndexCacheEntry, GraphSourceSignature};
