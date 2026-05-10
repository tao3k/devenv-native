//! Coordinates the Studio repo analysis index status flight branch and keeps its child modules behind one documented reasoning-tree boundary.

mod diagnostics;
mod encoding;
mod provider;

#[cfg(all(test, feature = "duckdb"))]
pub(crate) use diagnostics::configured_repo_index_status_diagnostics_engine_kind;
pub(crate) use diagnostics::repo_index_status_response_with_diagnostics;
#[cfg(test)]
pub(crate) use diagnostics::summarize_repo_index_status_diagnostics;
pub(crate) use encoding::{
    build_repo_index_status_flight_batch, build_repo_index_status_flight_metadata,
};
pub(crate) use provider::StudioRepoIndexStatusFlightRouteProvider;

#[cfg(test)]
#[path = "../../../../../../../tests/unit/gateway/studio/router/handlers/repo/analysis/index_status_flight.rs"]
mod tests;
