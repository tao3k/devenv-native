#[path = "projection.rs"]
pub(crate) mod projection;
#[path = "repo.rs"]
pub(crate) mod repo;
#[path = "resource.rs"]
pub(crate) mod resource;
#[path = "search.rs"]
pub(crate) mod search;
#[path = "sync.rs"]
pub(crate) mod sync;

#[cfg(test)]
#[path = "../../../../../../tests/unit/gateway/studio/router/handlers/repo/parse.rs"]
mod tests;
