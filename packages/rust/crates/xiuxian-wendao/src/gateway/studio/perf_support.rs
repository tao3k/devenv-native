#[path = "perf_support/fixture.rs"]
mod fixture;
#[path = "perf_support/git.rs"]
pub(crate) mod git;
#[path = "perf_support/root.rs"]
pub(crate) mod root;
#[path = "perf_support/state.rs"]
pub(crate) mod state;
#[cfg(test)]
#[path = "../../../tests/unit/gateway/studio/perf_support/mod.rs"]
mod tests;
#[path = "perf_support/workspace.rs"]
pub(crate) mod workspace;

#[cfg(feature = "julia")]
pub use fixture::prepare_gateway_perf_fixture_with_julia_parser_summary_transport;
pub use fixture::{
    GatewayPerfFixture, GatewayRepoIndexControllerDebugSnapshot, prepare_gateway_perf_fixture,
    prepare_gateway_real_workspace_perf_fixture,
};
