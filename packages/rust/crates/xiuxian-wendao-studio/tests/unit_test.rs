//! Cargo entry point for `xiuxian-wendao-studio` unit tests.

#[cfg(feature = "contracts")]
#[path = "unit/contracts_dependency_boundary/mod.rs"]
mod contracts_dependency_boundary;
#[cfg(feature = "contracts")]
#[path = "unit/contracts_routes.rs"]
mod contracts_routes;
#[cfg(feature = "contracts")]
#[path = "unit/contracts_types.rs"]
mod contracts_types;
#[path = "unit/namespace.rs"]
mod namespace;
#[cfg(feature = "document-extract-audio-shards")]
pub use xiuxian_wendao_studio::studio;
#[cfg(feature = "studio")]
#[path = "unit/studio_search_index_api.rs"]
mod studio_search_index_api;
#[cfg(feature = "document-extract-audio-shards")]
#[path = "unit/mod.rs"]
mod unit;
