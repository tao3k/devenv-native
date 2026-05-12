//! Runtime support for Wendao command binaries.

/// Command-line interface runtime for the main `wendao` binary.
#[cfg(feature = "cli-bin-support")]
#[path = "wendao.rs"]
pub mod wendao;

/// Runtime-backed Arrow Flight server entrypoint support.
#[cfg(feature = "zhenfa-router")]
#[path = "flight_server.rs"]
pub mod flight_server;

/// Runtime-backed FlightSQL server entrypoint support.
#[cfg(feature = "julia")]
#[path = "flightsql_server.rs"]
pub mod flightsql_server;
