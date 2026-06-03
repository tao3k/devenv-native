//! Arrow Flight service owner for Wendao query and rerank routes.

mod cache;
mod construction;
mod core;
mod internal_auth;
mod payload;
mod routing;
mod service;

#[cfg(test)]
#[path = "../../../../tests/unit/transport/server/flight/cache.rs"]
mod cache_tests;
#[cfg(test)]
#[path = "../../../../tests/unit/transport/server/flight/internal_auth.rs"]
mod internal_auth_tests;
#[cfg(test)]
#[path = "../../../../tests/unit/transport/server/flight/routing.rs"]
mod routing_tests;

pub use core::WendaoFlightService;
pub(super) use core::WendaoFlightService as ServiceCore;
pub use internal_auth::{WendaoFlightInternalSecurity, WendaoFlightInternalSecurityError};
