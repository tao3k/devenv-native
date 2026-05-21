//! Coordinates the Studio handlers repo parse branch and keeps its child modules behind one documented reasoning-tree boundary.

#[path = "projection.rs"]
pub(crate) mod projection;
#[path = "resource.rs"]
pub(crate) mod resource;
#[path = "search.rs"]
pub(crate) mod search;
#[path = "source.rs"]
pub(crate) mod source;
#[path = "sync.rs"]
pub(crate) mod sync;

#[cfg(test)]
#[path = "../../../../../../tests/unit/gateway/studio/router/handlers/repo/parse.rs"]
mod tests;
