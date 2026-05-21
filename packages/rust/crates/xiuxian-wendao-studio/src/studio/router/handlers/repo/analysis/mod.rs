//! Repository analysis endpoint handlers for Studio API.

#[path = "doc_coverage.rs"]
pub(crate) mod doc_coverage;
#[path = "flight.rs"]
pub(crate) mod flight;
#[path = "index_flight.rs"]
pub(crate) mod index_flight;
#[path = "index_status_flight/mod.rs"]
pub(crate) mod index_status_flight;
#[path = "overview.rs"]
pub(crate) mod overview;
#[path = "overview_flight.rs"]
pub(crate) mod overview_flight;
#[path = "projected_page_index_tree_flight.rs"]
pub(crate) mod projected_page_index_tree_flight;
#[path = "projected_retrieval_context_flight.rs"]
pub(crate) mod projected_retrieval_context_flight;
#[path = "refine_doc_flight.rs"]
pub(crate) mod refine_doc_flight;
#[path = "search/mod.rs"]
pub(crate) mod search;
#[path = "service/mod.rs"]
mod service;
#[path = "sync.rs"]
pub(crate) mod sync;
#[path = "sync_flight.rs"]
pub(crate) mod sync_flight;
