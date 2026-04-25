mod cold_start;
mod graph;
mod helpers;
mod lifecycle;
mod search;
mod types;
mod ui;

#[cfg(test)]
#[path = "../../../../../tests/unit/gateway/studio/router/state/mod.rs"]
mod tests;

#[cfg(any(test, feature = "performance"))]
pub(crate) use cold_start::StudioSearchColdStartTelemetryState;
pub use cold_start::{
    StudioSearchColdStartCorpusTelemetry, StudioSearchColdStartEvent,
    StudioSearchColdStartTelemetry,
};
#[cfg(test)]
pub(crate) use helpers::supported_code_kinds;
pub use types::{GatewayState, StudioBootstrapBackgroundIndexingTelemetry, StudioState};
#[cfg(test)]
pub(crate) use types::{GraphIndexCacheEntry, GraphSourceSignature};
