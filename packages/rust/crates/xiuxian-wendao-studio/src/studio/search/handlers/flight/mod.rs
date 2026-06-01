//! Coordinates the Studio search handlers flight branch and keeps its child modules behind one documented reasoning-tree boundary.

#[cfg(feature = "duckdb")]
mod dataset_ontology;
mod ontology_candidate_inspection;
mod provider;
mod repo_search;
mod service;
#[cfg(test)]
#[path = "../../../../../tests/unit/gateway/studio/search/handlers/flight/mod.rs"]
mod tests;

#[cfg(test)]
pub(crate) use self::ontology_candidate_inspection::candidate_inspection_report_batch;
#[cfg(feature = "flight-server-bin-support")]
pub(crate) use self::repo_search::build_studio_flight_service_for_roots_with_weights;
#[cfg(feature = "cli-bin-support")]
pub(crate) use self::repo_search::build_studio_flight_service_with_weights;
pub use self::repo_search::{
    StudioFlightRoots, StudioRepoSearchFlightRouteProvider, bootstrap_sample_repo_search_content,
    build_repo_search_flight_service, build_repo_search_flight_service_with_weights,
    build_studio_flight_service, build_studio_flight_service_for_roots,
};
#[cfg(test)]
pub(crate) use self::service::build_studio_search_flight_service_with_repo_provider;
