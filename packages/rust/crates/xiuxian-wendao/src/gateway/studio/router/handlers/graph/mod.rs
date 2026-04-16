//! Graph intelligence and visualization endpoints for Studio API.

pub(crate) mod flight;
#[cfg(test)]
pub(crate) mod neighbors;
mod service;
pub(crate) mod shared;
pub(crate) mod topology;
pub(crate) mod topology_flight;

#[cfg(test)]
#[path = "../../../../../../tests/unit/gateway/studio/router/handlers/graph/mod.rs"]
mod tests;
