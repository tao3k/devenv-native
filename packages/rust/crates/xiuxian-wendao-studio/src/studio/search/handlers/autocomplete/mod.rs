//! Coordinates the Studio search handlers autocomplete branch and keeps its child modules behind one documented reasoning-tree boundary.

mod batch;
mod provider;
mod response;

pub(crate) use provider::StudioAutocompleteFlightRouteProvider;
#[cfg(test)]
pub(crate) use response::build_autocomplete_response;
