mod fixture;
pub(crate) mod git;
pub(crate) mod root;
pub(crate) mod state;
#[cfg(test)]
#[path = "../../../../tests/unit/gateway/studio/perf_support/mod.rs"]
mod tests;
pub(crate) mod workspace;

#[cfg(feature = "julia")]
pub use fixture::prepare_gateway_perf_fixture_with_julia_parser_summary_transport;
pub use fixture::{
    GatewayPerfFixture, GatewayRepoIndexControllerDebugSnapshot, prepare_gateway_perf_fixture,
    prepare_gateway_real_workspace_perf_fixture,
};
