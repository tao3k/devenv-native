//! Coordinates the Studio search handlers knowledge branch and keeps its child modules behind one documented reasoning-tree boundary.

#[path = "intent/mod.rs"]
mod intent;
mod intent_policy;
mod merge;
mod response;

#[cfg(test)]
pub(crate) use intent::build_intent_cache_key;
#[cfg(test)]
pub use intent::build_intent_search_response;
#[cfg(test)]
pub(crate) use intent::ensure_intent_indices;
pub(crate) use intent::load_intent_search_flight_response;
#[cfg(test)]
pub(crate) use intent::load_intent_search_response_with_metadata;
pub(crate) use intent::{
    IntentSearchTransportMetadata, search_hit_batch_from_hits, search_response_flight_app_metadata,
};
#[cfg(test)]
pub(crate) use response::build_knowledge_search_response;
pub(crate) use response::load_knowledge_search_flight_response;
