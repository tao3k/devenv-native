//! Coordinates the Studio docs service projection branch and keeps its child modules behind one documented reasoning-tree boundary.

#[path = "family.rs"]
pub(crate) mod family;
#[path = "gap_report.rs"]
pub(crate) mod gap_report;
#[path = "navigation.rs"]
pub(crate) mod navigation;
#[path = "page.rs"]
pub(crate) mod page;
#[path = "retrieval.rs"]
pub(crate) mod retrieval;
#[path = "search.rs"]
pub(crate) mod search;
