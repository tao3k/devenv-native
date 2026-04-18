//! Repository analysis endpoint handlers for Studio API.

#[path = "analysis/doc_coverage.rs"]
pub(crate) mod doc_coverage;
#[path = "analysis/flight.rs"]
pub(crate) mod flight;
#[path = "analysis/index_flight.rs"]
pub(crate) mod index_flight;
#[path = "analysis/index_status_flight/mod.rs"]
pub(crate) mod index_status_flight;
#[path = "analysis/overview.rs"]
pub(crate) mod overview;
#[path = "analysis/overview_flight.rs"]
pub(crate) mod overview_flight;
#[path = "analysis/projected_page_index_tree_flight.rs"]
pub(crate) mod projected_page_index_tree_flight;
#[path = "analysis/refine_doc_flight.rs"]
pub(crate) mod refine_doc_flight;
#[path = "analysis/search.rs"]
pub(crate) mod search;
#[path = "analysis/service.rs"]
mod service;
#[path = "analysis/sync.rs"]
pub(crate) mod sync;
#[path = "analysis/sync_flight.rs"]
pub(crate) mod sync_flight;
