//! Arrow Flight service owner for Wendao runtime query and rerank routes.

mod cache;
mod construction;
mod core;
mod payload;
mod routing;
mod service;

pub use core::WendaoFlightService;
pub(super) use core::WendaoFlightService as ServiceCore;
