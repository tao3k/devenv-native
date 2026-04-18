//! Graph intelligence and visualization endpoints for Studio API.

#[path = "graph/flight.rs"]
pub(crate) mod flight;
#[cfg(test)]
#[path = "graph/neighbors.rs"]
pub(crate) mod neighbors;
#[path = "graph/service.rs"]
mod service;
#[path = "graph/shared/mod.rs"]
pub(crate) mod shared;
#[path = "graph/topology.rs"]
pub(crate) mod topology;
#[path = "graph/topology_flight.rs"]
pub(crate) mod topology_flight;

#[cfg(test)]
#[path = "../../../../../tests/unit/gateway/studio/router/handlers/graph/mod.rs"]
mod tests;
