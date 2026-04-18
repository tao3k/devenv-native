#[path = "parse/projection.rs"]
pub(crate) mod projection;
#[path = "parse/repo.rs"]
pub(crate) mod repo;
#[path = "parse/resource.rs"]
pub(crate) mod resource;
#[path = "parse/search.rs"]
pub(crate) mod search;
#[path = "parse/sync.rs"]
pub(crate) mod sync;

#[cfg(test)]
#[path = "../../../../../../tests/unit/gateway/studio/router/handlers/repo/parse.rs"]
mod tests;
