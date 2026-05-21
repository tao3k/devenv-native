//! Coordinates the Studio search handlers definition branch and keeps its child modules behind one documented reasoning-tree boundary.

mod batch;
mod path;
mod provider;
mod response;

pub(crate) use provider::StudioDefinitionFlightRouteProvider;
#[cfg(test)]
pub(crate) use response::build_definition_response;
