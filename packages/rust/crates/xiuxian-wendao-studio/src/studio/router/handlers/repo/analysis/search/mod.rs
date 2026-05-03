#[path = "cache.rs"]
mod cache;
#[path = "example.rs"]
pub(crate) mod example;
#[path = "import.rs"]
pub(crate) mod import;
#[path = "module.rs"]
pub(crate) mod module;
#[path = "publication.rs"]
mod publication;
#[path = "service/mod.rs"]
mod service;
#[path = "symbol.rs"]
pub(crate) mod symbol;

#[cfg(test)]
#[path = "../../../../../../../tests/unit/gateway/studio/router/handlers/repo/analysis/search/mod.rs"]
mod tests;
