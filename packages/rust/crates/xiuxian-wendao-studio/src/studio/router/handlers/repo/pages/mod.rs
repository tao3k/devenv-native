//! Coordinates the Studio handlers repo pages branch and keeps its child modules behind one documented reasoning-tree boundary.

#[path = "collection.rs"]
pub(crate) mod collection;
#[path = "page.rs"]
pub(crate) mod page;
#[path = "page_index.rs"]
pub(crate) mod page_index;
