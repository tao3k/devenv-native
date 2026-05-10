//! Coordinates the Studio handlers docs service branch and keeps its child modules behind one documented reasoning-tree boundary.

#[path = "planner.rs"]
pub(crate) mod planner;
#[path = "projection/mod.rs"]
pub(crate) mod projection;
#[path = "runtime.rs"]
mod runtime;
