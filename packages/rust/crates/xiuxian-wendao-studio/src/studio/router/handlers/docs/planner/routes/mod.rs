//! Coordinates the Studio docs planner routes branch and keeps its child modules behind one documented reasoning-tree boundary.

#[path = "item.rs"]
pub(crate) mod item;
#[path = "queue.rs"]
pub(crate) mod queue;
#[path = "rank.rs"]
pub(crate) mod rank;
#[path = "search.rs"]
pub(crate) mod search;
#[path = "workset.rs"]
pub(crate) mod workset;
