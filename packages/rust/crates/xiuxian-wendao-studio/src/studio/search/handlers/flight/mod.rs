mod provider;
mod repo_search;
mod service;
#[cfg(test)]
#[path = "../../../../../tests/unit/gateway/studio/search/handlers/flight/mod.rs"]
mod tests;

#[cfg(test)]
pub(crate) use self::provider::StudioSearchFlightRouteProvider;
pub use self::repo_search::{
    StudioRepoSearchFlightRouteProvider, bootstrap_sample_repo_search_content,
    build_repo_search_flight_service, build_repo_search_flight_service_with_weights,
    build_studio_flight_service, build_studio_flight_service_for_roots,
    build_studio_flight_service_for_roots_with_weights, build_studio_flight_service_with_weights,
};
#[cfg(test)]
pub(crate) use self::service::build_studio_search_flight_service_with_repo_provider;
