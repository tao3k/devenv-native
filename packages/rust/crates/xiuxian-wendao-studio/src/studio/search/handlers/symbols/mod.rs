//! Coordinates the Studio search handlers symbols branch and keeps its child modules behind one documented reasoning-tree boundary.

mod batch;
mod hit;
mod matcher;
mod response;

pub(crate) use response::load_symbol_search_flight_response;
#[cfg(test)]
pub(crate) use response::load_symbol_search_response;
