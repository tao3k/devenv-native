//! Coordinates the Studio router handlers docs branch and keeps its child modules behind one documented reasoning-tree boundary.

#[path = "planner/mod.rs"]
pub(crate) mod planner;
#[path = "projection/mod.rs"]
pub(crate) mod projection;
#[path = "service/mod.rs"]
mod service;
#[path = "types/mod.rs"]
mod types;
