//! Coordinates the Studio search handlers references branch and keeps its child modules behind one documented reasoning-tree boundary.

mod batch;
mod response;

pub(crate) use response::load_reference_search_flight_response;
#[cfg(test)]
pub(crate) use response::load_reference_search_response;
