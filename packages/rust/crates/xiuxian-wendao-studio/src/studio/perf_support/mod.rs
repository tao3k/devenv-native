//! Coordinates the Studio studio perf support branch and keeps its child modules behind one documented reasoning-tree boundary.

#[path = "fixture.rs"]
mod fixture;
#[path = "git.rs"]
pub(crate) mod git;
#[path = "root.rs"]
pub(crate) mod root;
#[path = "state.rs"]
pub(crate) mod state;
#[cfg(test)]
#[path = "../../../tests/unit/gateway/studio/perf_support/mod.rs"]
mod tests;
#[path = "workspace.rs"]
pub(crate) mod workspace;

#[cfg(feature = "julia")]
pub use fixture::prepare_gateway_perf_fixture_with_julia_parser_summary_transport;
pub use fixture::{
    GatewayPerfFixture, GatewayRepoIndexControllerDebugSnapshot, prepare_gateway_perf_fixture,
    prepare_gateway_real_workspace_perf_fixture,
};
