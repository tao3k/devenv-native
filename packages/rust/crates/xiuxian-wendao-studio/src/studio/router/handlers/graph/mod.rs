//! Graph intelligence and visualization endpoints for Studio API.

#[path = "flight.rs"]
pub(crate) mod flight;
#[cfg(test)]
#[path = "../../../../../tests/unit/gateway/studio/router/handlers/graph/neighbors_impl.rs"]
pub(crate) mod neighbors;
#[path = "service.rs"]
mod service;
#[path = "shared/mod.rs"]
pub(crate) mod shared;
#[path = "topology.rs"]
pub(crate) mod topology;
#[path = "topology_flight.rs"]
pub(crate) mod topology_flight;

#[cfg(test)]
#[path = "../../../../../tests/unit/gateway/studio/router/handlers/graph/mod.rs"]
mod tests;
