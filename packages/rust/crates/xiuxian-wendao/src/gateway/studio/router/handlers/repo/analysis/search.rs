#[path = "search/cache.rs"]
mod cache;
#[path = "search/example.rs"]
pub(crate) mod example;
#[path = "search/import.rs"]
pub(crate) mod import;
#[path = "search/module.rs"]
pub(crate) mod module;
#[path = "search/publication.rs"]
mod publication;
#[path = "search/service.rs"]
mod service;
#[path = "search/symbol.rs"]
pub(crate) mod symbol;

#[cfg(test)]
#[path = "../../../../../../../tests/unit/gateway/studio/router/handlers/repo/analysis/search/mod.rs"]
mod tests;
