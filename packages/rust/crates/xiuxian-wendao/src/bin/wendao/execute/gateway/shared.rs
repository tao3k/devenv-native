//! Shared gateway command state and constants.

use xiuxian_wendao::gateway::studio::GatewayState;

/// Shared state for the gateway server.
pub(crate) type AppState = GatewayState;

/// Default port for the gateway server.
pub(crate) const DEFAULT_PORT: u16 = 9517;

/// Default bind address for the gateway server.
pub(crate) const DEFAULT_BIND_ADDR: [u8; 4] = [127, 0, 0, 1];

/// Environment variable that points at the pidfile owned by the managed gateway process.
pub(crate) const GATEWAY_PIDFILE_ENV: &str = "WENDAO_GATEWAY_PIDFILE";

/// Response header that exposes the current gateway process id to readiness probes.
pub(crate) const GATEWAY_PROCESS_ID_HEADER: &str = "x-wendao-process-id";
