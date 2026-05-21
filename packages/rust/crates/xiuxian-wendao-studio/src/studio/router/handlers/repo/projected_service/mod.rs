//! Coordinates the Studio handlers repo projected service branch and keeps its child modules behind one documented reasoning-tree boundary.

#[path = "analysis.rs"]
pub(crate) mod analysis;
#[path = "collection.rs"]
pub(crate) mod collection;
#[path = "family.rs"]
pub(crate) mod family;
#[path = "pages.rs"]
pub(crate) mod pages;
#[path = "retrieval.rs"]
pub(crate) mod retrieval;
