//! Coordinates the Studio handlers knowledge intent branch and keeps its child modules behind one documented reasoning-tree boundary.

mod cache;
mod entry;
mod flight;
mod indices;
mod response;
mod sources;
mod types;

#[cfg(test)]
pub(crate) use cache::build_intent_cache_key;
#[cfg(test)]
pub use entry::build_intent_search_response;
#[cfg(test)]
pub(crate) use entry::load_intent_search_response_with_metadata;
pub(crate) use flight::{
    load_intent_search_flight_response, search_hit_batch_from_hits,
    search_response_flight_app_metadata,
};
#[cfg(test)]
pub(crate) use indices::ensure_intent_indices;
pub(crate) use types::IntentSearchTransportMetadata;
#[cfg(all(test, feature = "duckdb"))]
pub(crate) use types::configured_parquet_query_engine_label;
